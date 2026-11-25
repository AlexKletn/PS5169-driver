use crate::{Error, Mode, Orientation, Profile, RegisterIo, route::Route};

const CHIP_ID: u16 = 0x6987;
const CHIP_ID_LOW: u8 = 0xac;
const CHIP_ID_HIGH: u8 = 0xad;
const RESET: u8 = 0x9d;
const ROUTE: u8 = 0x40;

pub struct Ps5169<Bus, Settings> {
    io: Bus,
    profile: Profile<Settings>,
    mode: Mode,
    orientation: Orientation,
    applied: Option<Route>,
}

impl<Bus: RegisterIo, Settings: AsRef<[u8]>> Ps5169<Bus, Settings> {
    pub fn new(io: Bus, profile: Profile<Settings>) -> Self {
        Self {
            io,
            profile,
            mode: Mode::Safe,
            orientation: Orientation::Detached,
            applied: None,
        }
    }

    pub fn initialize(&mut self) -> Result<(), Error<Bus::Error>> {
        self.applied = None;
        let low = self.read_chip_id_byte(CHIP_ID_LOW)?;
        let high = self.read_chip_id_byte(CHIP_ID_HIGH)?;
        let id = u16::from_le_bytes([low, high]);
        if id != CHIP_ID {
            return Err(Error::UnexpectedChipId(id));
        }

        self.write(RESET, 0x80)?;
        self.io.wait_milliseconds(10);
        Self::write_all(&mut self.io, self.profile.initial())?;
        self.write(ROUTE, Route::OFF.register_value())?;
        self.applied = Some(Route::OFF);
        Ok(())
    }

    pub fn set_orientation(&mut self, orientation: Orientation) -> Result<(), Error<Bus::Error>> {
        let mode = match (self.orientation, orientation) {
            (_, Orientation::Detached) => Mode::Safe,
            (Orientation::Detached, _) => Mode::Usb,
            _ => self.mode,
        };
        self.apply(mode, orientation)
    }

    pub fn set_mode(&mut self, mode: Mode) -> Result<(), Error<Bus::Error>> {
        self.apply(mode, self.orientation)
    }

    pub fn resume(&mut self) -> Result<(), Error<Bus::Error>> {
        let applied = self.applied.take();
        if let Some(route) = applied {
            let (register, expected) = self.profile.retention_check();
            if self.read(ROUTE)? == route.register_value() && self.read(register)? == expected {
                self.applied = applied;
                return Ok(());
            }
        }
        self.initialize()?;
        self.apply(self.mode, self.orientation)
    }

    fn apply(&mut self, mode: Mode, orientation: Orientation) -> Result<(), Error<Bus::Error>> {
        self.mode = mode;
        self.orientation = orientation;
        let route = Route::new(mode, orientation);
        if self.applied == Some(route) {
            return Ok(());
        }

        let previous = self.applied.take().map(|route| route.mode);
        if previous != Some(Mode::Safe) {
            self.write(ROUTE, Route::OFF.register_value())?;
            Self::write_all(&mut self.io, self.profile.disable(previous))?;
        }

        if route != Route::OFF {
            self.write(ROUTE, route.register_value())?;
            Self::write_all(&mut self.io, self.profile.enable(route.mode))?;
        }
        self.applied = Some(route);
        Ok(())
    }

    fn read_chip_id_byte(&mut self, address: u8) -> Result<u8, Error<Bus::Error>> {
        self.io
            .read(address)
            .map_err(|cause| Error::ReadChipId { address, cause })
    }

    fn read(&mut self, address: u8) -> Result<u8, Error<Bus::Error>> {
        self.io
            .read(address)
            .map_err(|cause| Error::Register { address, cause })
    }

    fn write(&mut self, address: u8, value: u8) -> Result<(), Error<Bus::Error>> {
        self.io
            .write(address, value)
            .map_err(|cause| Error::Register { address, cause })
    }

    fn write_all(io: &mut Bus, settings: &[[u8; 2]]) -> Result<(), Error<Bus::Error>> {
        for &[address, value] in settings {
            io.write(address, value)
                .map_err(|cause| Error::Register { address, cause })?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "chip.tests.rs"]
mod tests;
