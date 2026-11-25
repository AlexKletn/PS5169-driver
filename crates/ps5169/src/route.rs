use crate::{Mode, Orientation};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Route {
    pub(crate) mode: Mode,
    pub(crate) orientation: Orientation,
}

impl Route {
    pub(crate) const OFF: Self = Self {
        mode: Mode::Safe,
        orientation: Orientation::Detached,
    };

    pub(crate) fn new(mode: Mode, orientation: Orientation) -> Self {
        match (mode, orientation) {
            (Mode::Safe, _) | (_, Orientation::Detached) => Self::OFF,
            _ => Self { mode, orientation },
        }
    }

    pub(crate) fn register_value(self) -> u8 {
        let mode = match self.mode {
            Mode::Safe => 0x80,
            Mode::Usb => 0xc0,
            Mode::DisplayPort => 0xa0,
            Mode::UsbAndDisplayPort => 0xe0,
        };
        mode | if self.orientation == Orientation::Reverse {
            0x10
        } else {
            0
        }
    }
}
