#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error<BusError> {
    ReadChipId { address: u8, cause: BusError },
    UnexpectedChipId(u16),
    Register { address: u8, cause: BusError },
}
