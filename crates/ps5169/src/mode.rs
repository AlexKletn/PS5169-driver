#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Safe,
    Usb,
    DisplayPort,
    UsbAndDisplayPort,
}
