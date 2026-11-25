extern crate std;

use std::vec::Vec;

use super::*;
use crate::RegisterSequences;

struct TestBus {
    registers: [u8; 256],
    writes: Vec<(u8, u8)>,
    waits: Vec<u32>,
    fail_write: Option<u8>,
    fail_read: Option<u8>,
}

impl TestBus {
    fn new() -> Self {
        let mut registers = [0; 256];
        registers[0xac] = 0x87;
        registers[0xad] = 0x69;
        Self {
            registers,
            writes: Vec::new(),
            waits: Vec::new(),
            fail_write: None,
            fail_read: None,
        }
    }
}

impl RegisterIo for TestBus {
    type Error = ();

    fn read(&mut self, register: u8) -> Result<u8, ()> {
        if self.fail_read == Some(register) {
            return Err(());
        }
        Ok(self.registers[usize::from(register)])
    }

    fn write(&mut self, register: u8, value: u8) -> Result<(), ()> {
        if self.fail_write == Some(register) {
            return Err(());
        }
        self.registers[usize::from(register)] = value;
        self.writes.push((register, value));
        Ok(())
    }

    fn wait_milliseconds(&mut self, milliseconds: u32) {
        self.waits.push(milliseconds);
    }
}

fn sequences() -> RegisterSequences<&'static [u8]> {
    RegisterSequences {
        initial: &[0x9d, 0, 0x51, 0x46],
        usb_enable: &[0x8d, 1, 0x90, 1],
        usb_disable: &[0x8d, 0, 0x90, 0],
        dp_enable: &[0xa0, 0, 0xa1, 4],
        dp_disable: &[0xa0, 2, 0xa1, 0],
        mixed_enable: &[0x8d, 1, 0x90, 1, 0xa0, 0, 0xa1, 4],
        mixed_disable: &[0xa0, 2, 0x8d, 0, 0x90, 0, 0xa1, 0],
    }
}

fn chip() -> Ps5169<TestBus, &'static [u8]> {
    Ps5169::new(TestBus::new(), Profile::new(sequences()).unwrap())
}

#[test]
fn initializes_in_order_with_supplied_equalization() {
    let mut device = chip();
    device.initialize().unwrap();
    assert_eq!(device.io.waits, [10]);
    assert_eq!(
        device.io.writes,
        [(0x9d, 0x80), (0x9d, 0), (0x51, 0x46), (0x40, 0x80)]
    );
}

#[test]
fn rejects_unknown_chip_before_writing_registers() {
    let mut device = chip();
    device.io.registers[0xad] = 0;
    assert_eq!(device.initialize(), Err(Error::UnexpectedChipId(0x0087)));
    assert!(device.io.writes.is_empty());
}

#[test]
fn preserves_the_mode_on_orientation_changes_and_disables_on_detach() {
    let mut device = chip();
    device.initialize().unwrap();
    device.set_orientation(Orientation::Normal).unwrap();
    assert_eq!(device.io.registers[0x40], 0xc0);
    device.set_mode(Mode::UsbAndDisplayPort).unwrap();
    device.set_orientation(Orientation::Reverse).unwrap();
    assert_eq!(device.io.registers[0x40], 0xf0);
    device.set_orientation(Orientation::Detached).unwrap();
    assert_eq!(device.io.registers[0x40], 0x80);
    assert_eq!(device.io.registers[0x8d], 0);
}

#[test]
fn retries_from_the_safe_route_after_a_failed_write() {
    let mut device = chip();
    device.initialize().unwrap();
    device.io.fail_write = Some(0x90);
    assert_eq!(
        device.set_orientation(Orientation::Normal),
        Err(Error::Register {
            address: 0x90,
            cause: ()
        })
    );
    device.io.fail_write = None;
    device.io.writes.clear();
    device.set_orientation(Orientation::Normal).unwrap();
    assert_eq!(device.io.writes[0], (0x40, 0x80));
    assert_eq!(device.io.registers[0x90], 1);
}

#[test]
fn restores_a_route_only_if_registers_lost_the_profile() {
    let mut device = chip();
    device.initialize().unwrap();
    device.set_orientation(Orientation::Reverse).unwrap();
    device.set_mode(Mode::DisplayPort).unwrap();
    device.io.writes.clear();
    device.resume().unwrap();
    assert!(device.io.writes.is_empty());
    device.io.registers[0x51] = 0;
    device.resume().unwrap();
    assert_eq!(device.io.registers[0x51], 0x46);
    assert_eq!(device.io.registers[0x40], 0xb0);
}

#[test]
fn rejects_incomplete_profiles() {
    assert!(
        Profile::<&[u8]>::new(RegisterSequences {
            dp_disable: &[0x51],
            ..sequences()
        })
        .is_none()
    );
    assert!(
        Profile::<&[u8]>::new(RegisterSequences {
            mixed_enable: &[],
            ..sequences()
        })
        .is_none()
    );
    assert!(
        Profile::<&[u8]>::new(RegisterSequences {
            initial: &[0x40, 0x80],
            ..sequences()
        })
        .is_none()
    );
}

#[test]
fn reports_which_chip_id_register_failed_before_reset() {
    for address in [0xac, 0xad] {
        let mut device = chip();
        device.io.fail_read = Some(address);
        assert_eq!(
            device.initialize(),
            Err(Error::ReadChipId { address, cause: () })
        );
        assert!(device.io.writes.is_empty());
        assert!(device.io.waits.is_empty());
    }
}

#[test]
fn applies_mixed_disable_and_usb_enable_in_profile_order() {
    let mut device = chip();
    device.initialize().unwrap();
    device.set_orientation(Orientation::Normal).unwrap();
    device.set_mode(Mode::UsbAndDisplayPort).unwrap();
    device.io.writes.clear();

    device.set_mode(Mode::Usb).unwrap();

    assert_eq!(
        device.io.writes,
        [
            (0x40, 0x80),
            (0xa0, 2),
            (0x8d, 0),
            (0x90, 0),
            (0xa1, 0),
            (0x40, 0xc0),
            (0x8d, 1),
            (0x90, 1),
        ]
    );
}
