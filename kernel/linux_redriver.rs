// SPDX-License-Identifier: GPL-2.0-only

use core::{pin::Pin, ptr, ptr::NonNull};
use kernel::{
    alloc::KVec,
    bindings as b,
    device::Core,
    error::from_err_ptr,
    i2c, of,
    prelude::*,
    sync::global_lock,
    time::{Instant, Monotonic},
};

use crate::{
    Mode, Orientation, Ps5169, board_profile::BoardProfile, i2c_bus::I2cBus,
    registered_port::RegisteredPort,
};

global_lock! {
    // SAFETY: Initialized once before I2C driver registration.
    pub(crate) unsafe(uninit) static PORTS: Mutex<KVec<RegisteredPort>> = KVec::new();
}

fn set_orientation(port: &mut RegisteredPort, orientation: b::typec_orientation) -> Result {
    let orientation = match orientation {
        b::typec_orientation_TYPEC_ORIENTATION_NONE => Orientation::Detached,
        b::typec_orientation_TYPEC_ORIENTATION_NORMAL => Orientation::Normal,
        b::typec_orientation_TYPEC_ORIENTATION_REVERSE => Orientation::Reverse,
        _ => return Err(EINVAL),
    };
    port.chip.set_orientation(orientation).map_err(driver_error)
}

extern "C" fn orientation_callback(
    switch: *mut b::typec_switch_dev,
    orientation: b::typec_orientation,
) -> i32 {
    let result = PORTS
        .lock()
        .iter_mut()
        .find(|port| port.switch == switch.addr())
        .ok_or(ENODEV)
        .and_then(|port| set_orientation(port, orientation));
    result.map_or_else(Error::to_errno, |()| 0)
}

fn typec_mode(state: &b::typec_retimer_state) -> Result<Mode> {
    match state.mode as u32 {
        b::TYPEC_STATE_USB => Ok(Mode::Usb),
        b::TYPEC_STATE_SAFE => Ok(Mode::Safe),
        pin => {
            // SAFETY: The Type-C core keeps a non-null Alt Mode object alive during the callback.
            let alt = unsafe { state.alt.as_ref() }.ok_or(EOPNOTSUPP)?;
            if u32::from(alt.svid) != b::USB_TYPEC_DP_SID {
                return Err(EOPNOTSUPP);
            }
            match pin {
                b::TYPEC_DP_STATE_C | b::TYPEC_DP_STATE_E => Ok(Mode::DisplayPort),
                b::TYPEC_DP_STATE_D | b::TYPEC_DP_STATE_F => Ok(Mode::UsbAndDisplayPort),
                _ => Err(EOPNOTSUPP),
            }
        }
    }
}

extern "C" fn mode_callback(
    retimer: *mut b::typec_retimer,
    state: *mut b::typec_retimer_state,
) -> i32 {
    // SAFETY: The Type-C core passes a live state pointer for the duration of the callback.
    let state = unsafe { &*state };
    let result = typec_mode(state).and_then(|mode| {
        PORTS
            .lock()
            .iter_mut()
            .find(|port| port.retimer == retimer.addr())
            .ok_or(ENODEV)?
            .chip
            .set_mode(mode)
            .map_err(driver_error)
    });
    result.map_or_else(Error::to_errno, |()| 0)
}

#[cfg(CONFIG_PM_SLEEP)]
extern "C" fn resume(dev: *mut b::device) -> i32 {
    PORTS
        .lock()
        .iter_mut()
        .find(|port| port.device == dev.addr())
        .ok_or(ENODEV)
        .and_then(|port| port.chip.resume().map_err(driver_error))
        .map_or_else(Error::to_errno, |()| 0)
}

pub(crate) struct LinuxRedriver {
    device: usize,
    switch: *mut b::typec_switch_dev,
    retimer: *mut b::typec_retimer,
}

// SAFETY: This value owns the registrations. The global registry serializes callbacks and removal.
unsafe impl Send for LinuxRedriver {}

kernel::of_device_table!(
    OF_TABLE,
    <LinuxRedriver as i2c::Driver>::IdInfo,
    [(of::DeviceId::new(c"parade,ps5169"), ())]
);

impl i2c::Driver for LinuxRedriver {
    type IdInfo = ();
    type Data<'bound> = Self;
    const OF_ID_TABLE: Option<of::IdTable<Self::IdInfo>> = Some(&OF_TABLE);

    #[cfg(CONFIG_PM_SLEEP)]
    const PM_OPS: Option<&'static b::dev_pm_ops> = Some(&b::dev_pm_ops {
        resume: Some(resume),
        restore: Some(resume),
        // SAFETY: Unused PM callbacks are nullable function pointers.
        ..unsafe { core::mem::zeroed() }
    });

    fn probe<'bound>(
        client: &'bound i2c::I2cClient<Core<'_>>,
        _: Option<&'bound ()>,
    ) -> impl PinInit<Self, Error> + 'bound {
        Self::register(client)
    }

    fn unbind<'bound>(_: &'bound i2c::I2cClient<Core<'_>>, this: Pin<&Self>) {
        let mut ports = PORTS.lock();
        if let Some(index) = ports.iter().position(|port| port.device == this.device) {
            if let Ok(mut port) = ports.remove(index) {
                if let Err(error) = port.chip.set_orientation(Orientation::Detached) {
                    pr_err!("PS5169 could not disable the signal path: {error:?}\n");
                }
            }
        }
        drop(ports);
        // SAFETY: This bound driver owns both successful Type-C registrations.
        unsafe {
            b::typec_switch_unregister(this.switch);
            b::typec_retimer_unregister(this.retimer);
        }
    }
}

impl LinuxRedriver {
    fn register(client: &i2c::I2cClient<Core<'_>>) -> Result<Self> {
        let profile = BoardProfile::read(client.as_ref())?;
        let raw = client.as_raw();
        let mut chip = Ps5169::new(I2cBus(NonNull::new(raw).ok_or(ENODEV)?), profile.settings);
        let started = Instant::<Monotonic>::now();
        let device = client.as_ref();
        chip.initialize().map_err(|error| {
            let elapsed_ms = started.elapsed().as_millis_ceil();
            match error {
                crate::Error::ReadChipId { address, cause } => {
                    // TODO: Identify the startup dependency that causes PS5169 ID reads to time out.
                    if profile.defer_id_timeout && cause == ETIMEDOUT {
                        // SAFETY: Probe holds the live device and the arguments match the C format.
                        unsafe {
                            b::dev_err_probe(
                                ptr::addr_of!((*raw).dev),
                                EPROBE_DEFER.to_errno(),
                                c"PS5169 ID register 0x%02x timed out after %lld ms; deferring probe\n".as_char_ptr(),
                                u32::from(address),
                                elapsed_ms,
                            );
                        }
                        EPROBE_DEFER
                    } else {
                        dev_err!(device, "PS5169 ID register {address:#04x} read failed after {elapsed_ms} ms: {cause:?}\n");
                        cause
                    }
                }
                other => {
                    dev_err!(device, "PS5169 initialization failed after {elapsed_ms} ms: {other:?}\n");
                    driver_error(other)
                }
            }
        })?;
        dev_info!(
            device,
            "PS5169 initialized in {} ms\n",
            started.elapsed().as_millis_ceil()
        );

        let mut ports = PORTS.lock();
        ports.reserve(1, GFP_KERNEL)?;
        // SAFETY: Probe has a live client; the parent and firmware node outlive registration.
        let (dev, fwnode) = unsafe { (ptr::addr_of_mut!((*raw).dev), (*raw).dev.fwnode) };
        let retimer_desc = b::typec_retimer_desc {
            fwnode,
            set: Some(mode_callback),
            name: ptr::null(),
            drvdata: ptr::null_mut(),
        };
        let switch_desc = b::typec_switch_desc {
            fwnode,
            set: Some(orientation_callback),
            name: ptr::null(),
            drvdata: ptr::null_mut(),
        };
        // SAFETY: The Type-C core copies each descriptor and retains the live parent device.
        let retimer = from_err_ptr(unsafe { b::typec_retimer_register(dev, &retimer_desc) })?;
        // SAFETY: The firmware node and parent remain live through probe.
        let switch = match from_err_ptr(unsafe { b::typec_switch_register(dev, &switch_desc) }) {
            Ok(switch) => switch,
            Err(error) => {
                drop(ports);
                // SAFETY: Probe owns the retimer registration created above.
                unsafe { b::typec_retimer_unregister(retimer) };
                return Err(error);
            }
        };
        let device = dev.addr();
        if ports
            .push_within_capacity(RegisteredPort {
                chip,
                device,
                switch: switch.addr(),
                retimer: retimer.addr(),
            })
            .is_err()
        {
            drop(ports);
            // SAFETY: Probe owns the two registrations and has yet to publish a port.
            unsafe {
                b::typec_switch_unregister(switch);
                b::typec_retimer_unregister(retimer);
            }
            return Err(ENOMEM);
        }
        Ok(Self {
            device,
            switch,
            retimer,
        })
    }
}

fn driver_error(error: crate::Error<Error>) -> Error {
    match error {
        crate::Error::ReadChipId { cause, .. } | crate::Error::Register { cause, .. } => cause,
        crate::Error::UnexpectedChipId(_) => ENODEV,
    }
}
