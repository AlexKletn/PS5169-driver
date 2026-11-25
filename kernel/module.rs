// SPDX-License-Identifier: GPL-2.0-only

//! Linux I²C and Type-C integration for the PS5169 redriver.

#![allow(unreachable_pub)]
#[path = "../crates/ps5169/src/chip.rs"]
mod chip;
#[path = "../crates/ps5169/src/error.rs"]
mod error;
#[path = "../crates/ps5169/src/mode.rs"]
mod mode;
#[path = "../crates/ps5169/src/orientation.rs"]
mod orientation;
#[path = "../crates/ps5169/src/profile.rs"]
mod profile;
#[path = "../crates/ps5169/src/profile_data.rs"]
mod profile_data;
#[path = "../crates/ps5169/src/register_io.rs"]
mod register_io;
#[path = "../crates/ps5169/src/register_sequences.rs"]
mod register_sequences;
#[path = "../crates/ps5169/src/route.rs"]
mod route;

mod board_profile;
mod i2c_bus;
mod linux_redriver;
mod registered_port;

pub(crate) use {
    chip::Ps5169, error::Error, mode::Mode, orientation::Orientation, profile::Profile,
    register_io::RegisterIo, register_sequences::RegisterSequences,
};

use kernel::{InPlaceModule, ModuleMetadata, driver, i2c, prelude::*};
use linux_redriver::{LinuxRedriver, PORTS};

#[pin_data]
struct Ps5169Module {
    #[pin]
    _driver: driver::Registration<i2c::Adapter<LinuxRedriver>>,
}

impl InPlaceModule for Ps5169Module {
    fn init(module: &'static ThisModule) -> impl PinInit<Self, kernel::error::Error> {
        // SAFETY: Module initialization runs once before the driver publishes its callbacks.
        unsafe { PORTS.init() };
        try_pin_init!(Self {
            _driver <- driver::Registration::new(<Self as ModuleMetadata>::NAME, module),
        })
    }
}

module! {
    type: Ps5169Module,
    name: "ps5169",
    description: "Parade PS5169 USB-C redriver",
    license: "GPL",
}
