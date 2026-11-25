// SPDX-License-Identifier: GPL-2.0-only

use kernel::{device::Device, prelude::*};

use crate::{Profile, profile_data::SELECTED_SEQUENCES};

pub(crate) struct BoardProfile {
    pub(crate) settings: Profile<&'static [u8]>,
    pub(crate) defer_id_timeout: bool,
}

impl BoardProfile {
    pub(crate) fn read(device: &Device) -> Result<Self> {
        let node = device.fwnode().ok_or(ENODEV)?;
        let sequences = SELECTED_SEQUENCES.ok_or_else(|| {
            dev_err!(device, "No PS5169 profile selected at build time\n");
            ENODEV
        })?;
        Ok(Self {
            settings: Profile::new(sequences).ok_or(EINVAL)?,
            defer_id_timeout: node.property_read_bool(c"parade,defer-id-timeout"),
        })
    }
}
