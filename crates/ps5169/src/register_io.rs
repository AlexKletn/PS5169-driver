pub trait RegisterIo {
    type Error;

    fn read(&mut self, register: u8) -> Result<u8, Self::Error>;
    fn write(&mut self, register: u8, value: u8) -> Result<(), Self::Error>;
    fn wait_milliseconds(&mut self, milliseconds: u32);
}
