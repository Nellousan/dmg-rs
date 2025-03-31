#[derive(Default, Debug, Clone, Copy)]
pub enum ClockType {
    #[default]
    Zero = 256 * 4,
    One = 4 * 4,
    Two = 16 * 4,
    Three = 64 * 4,
}

impl From<u8> for ClockType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Zero,
            0x01 => Self::One,
            0x02 => Self::Two,
            0x03 => Self::Three,
            _ => unreachable!(),
        }
    }
}

impl From<ClockType> for u8 {
    fn from(value: ClockType) -> Self {
        match value {
            ClockType::Zero => 0x00,
            ClockType::One => 0x01,
            ClockType::Two => 0x02,
            ClockType::Three => 0x03,
        }
    }
}

#[derive(Default, Debug)]
pub struct Timer {
    div_register: u8,
    counter: u8,
    modulo: u8,
    enable: bool,
    selected_clock: ClockType,
    tick_counter: usize,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn tick(&mut self) -> bool {
        let mut raise_interrupt = false;
        self.tick_counter = self.tick_counter.wrapping_add(1usize);
        if self.tick_counter % 256 == 0 {
            self.div_register = self.div_register.wrapping_add(1);
        }
        if self.enable && self.tick_counter % self.selected_clock as usize == 0 {
            let value = self.counter.checked_add(1);
            if let Some(val) = value {
                self.counter = val;
            } else {
                self.counter = self.modulo;
                raise_interrupt = true;
            }
        }
        raise_interrupt
    }

    fn write_timer_control(&mut self, value: u8) {
        self.enable = match value & 0x04 {
            0 => false,
            _ => true,
        };

        self.selected_clock = ClockType::from(value & 0x03);
    }

    pub fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => self.div_register = 0u8,
            0xFF05 => self.counter = value,
            0xFF06 => self.modulo = value,
            0xFF07 => self.write_timer_control(value),
            _ => unreachable!(),
        }
    }

    fn read_timer_control(&self) -> u8 {
        let enable = match self.enable {
            true => 1,
            false => 0,
        };

        let clock: u8 = self.selected_clock.into();
        (enable << 2) | clock
    }

    pub fn read_8(&self, address: u16) -> u8 {
        match address {
            0xFF04 => self.div_register,
            0xFF05 => self.counter,
            0xFF06 => self.modulo,
            0xFF07 => self.read_timer_control(),
            _ => unreachable!(),
        }
    }
}
