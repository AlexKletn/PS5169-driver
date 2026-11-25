// SPDX-License-Identifier: GPL-2.0-only

use core::ptr::NonNull;
use kernel::{
    bindings,
    device::Device,
    prelude::*,
    time::{Delta, Instant, Monotonic, delay::fsleep},
};

use crate::RegisterIo;

pub(crate) struct I2cBus(pub(crate) NonNull<bindings::i2c_client>);

// SAFETY: The port registry serializes access and removes the bus before the client is freed.
unsafe impl Send for I2cBus {}

impl RegisterIo for I2cBus {
    type Error = Error;

    fn read(&mut self, register: u8) -> Result<u8> {
        // SAFETY: The adapter holds a live client during probe or under the port registry lock.
        let result = unsafe { bindings::i2c_smbus_read_byte_data(self.0.as_ptr(), register) };
        u8::try_from(result).map_err(|_| {
            if result < 0 {
                Error::from_errno(result)
            } else {
                EIO
            }
        })
    }

    fn write(&mut self, register: u8, value: u8) -> Result {
        // SAFETY: The adapter holds a live client during probe or under the port registry lock.
        kernel::error::to_result(unsafe {
            bindings::i2c_smbus_write_byte_data(self.0.as_ptr(), register, value)
        })
    }

    fn wait_milliseconds(&mut self, milliseconds: u32) {
        let started = Instant::<Monotonic>::now();
        fsleep(Delta::from_millis(milliseconds.into()));
        // SAFETY: The I²C client remains live during probe or under the port registry lock.
        let device: &Device =
            unsafe { Device::from_raw(core::ptr::addr_of_mut!((*self.0.as_ptr()).dev)) };
        dev_info!(
            device,
            "PS5169 reset wait: requested {milliseconds} ms, elapsed {} us\n",
            started.elapsed().as_micros_ceil()
        );
    }
}
