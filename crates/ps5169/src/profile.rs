use crate::{Mode, RegisterSequences};

const USB_RX_EQUALIZATION: u8 = 0x51;

pub struct Profile<Settings> {
    settings: RegisterSequences<Settings>,
    usb_rx_equalization: u8,
}

impl<Settings: AsRef<[u8]>> Profile<Settings> {
    pub fn new(settings: RegisterSequences<Settings>) -> Option<Self> {
        for sequence in [
            &settings.initial,
            &settings.usb_enable,
            &settings.usb_disable,
            &settings.dp_enable,
            &settings.dp_disable,
            &settings.mixed_enable,
            &settings.mixed_disable,
        ] {
            let bytes = sequence.as_ref();
            if bytes.is_empty() || !bytes.len().is_multiple_of(2) {
                return None;
            }
        }

        let usb_rx_equalization = *pairs(&settings.initial)
            .iter()
            .rev()
            .find_map(|[register, value]| (*register == USB_RX_EQUALIZATION).then_some(value))?;
        Some(Self {
            settings,
            usb_rx_equalization,
        })
    }

    pub(crate) fn initial(&self) -> &[[u8; 2]] {
        pairs(&self.settings.initial)
    }

    pub(crate) fn enable(&self, mode: Mode) -> &[[u8; 2]] {
        match mode {
            Mode::Usb => pairs(&self.settings.usb_enable),
            Mode::DisplayPort => pairs(&self.settings.dp_enable),
            Mode::UsbAndDisplayPort => pairs(&self.settings.mixed_enable),
            Mode::Safe => &[],
        }
    }

    pub(crate) fn disable(&self, previous: Option<Mode>) -> &[[u8; 2]] {
        match previous {
            Some(Mode::Usb) => pairs(&self.settings.usb_disable),
            Some(Mode::DisplayPort) => pairs(&self.settings.dp_disable),
            Some(Mode::Safe) => &[],
            Some(Mode::UsbAndDisplayPort) | None => pairs(&self.settings.mixed_disable),
        }
    }

    pub(crate) fn retention_check(&self) -> (u8, u8) {
        (USB_RX_EQUALIZATION, self.usb_rx_equalization)
    }
}

fn pairs(settings: &impl AsRef<[u8]>) -> &[[u8; 2]] {
    settings.as_ref().as_chunks::<2>().0
}
