// SPDX-License-Identifier: GPL-2.0-only

use crate::{Ps5169, i2c_bus::I2cBus};

pub(crate) struct RegisteredPort {
    pub(crate) chip: Ps5169<I2cBus, &'static [u8]>,
    pub(crate) device: usize,
    pub(crate) switch: usize,
    pub(crate) retimer: usize,
}
