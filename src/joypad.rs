use tracing::debug;

use crate::thread::DmgButton;

#[derive(Debug)]
pub enum SelectMode {
    Buttons,
    DirectionalPad,
    Other,
}

#[derive(Debug)]
pub struct Joypad {
    buttons: u8,
    d_pad: u8,
    select_mode: SelectMode,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            buttons: 0xFu8,
            d_pad: 0xFu8,
            select_mode: SelectMode::Other,
        }
    }

    pub fn button_pressed(&mut self, button: DmgButton) {
        let clear = |target: &mut u8, n: u8| {
            *target &= !(1u8 << n);
        };
        match button {
            DmgButton::Up => clear(&mut self.d_pad, 2),
            DmgButton::Down => clear(&mut self.d_pad, 3),
            DmgButton::Left => clear(&mut self.d_pad, 1),
            DmgButton::Right => clear(&mut self.d_pad, 0),
            DmgButton::A => clear(&mut self.buttons, 0),
            DmgButton::B => clear(&mut self.buttons, 1),
            DmgButton::Start => clear(&mut self.buttons, 3),
            DmgButton::Select => clear(&mut self.buttons, 2),
        }
    }

    pub fn button_released(&mut self, button: DmgButton) {
        let set = |target: &mut u8, n: u8| {
            *target |= 1u8 << n;
        };
        match button {
            DmgButton::Up => set(&mut self.d_pad, 2),
            DmgButton::Down => set(&mut self.d_pad, 3),
            DmgButton::Left => set(&mut self.d_pad, 1),
            DmgButton::Right => set(&mut self.d_pad, 0),
            DmgButton::A => set(&mut self.buttons, 0),
            DmgButton::B => set(&mut self.buttons, 1),
            DmgButton::Start => set(&mut self.buttons, 3),
            DmgButton::Select => set(&mut self.buttons, 2),
        }
    }

    pub fn write(&mut self, value: u8) {
        let value = (value >> 4) & 0x03;
        match value {
            0x01 => self.select_mode = SelectMode::DirectionalPad,
            0x02 => self.select_mode = SelectMode::Buttons,
            0x03 => self.select_mode = SelectMode::Other,
            _ => unreachable!(),
        }
    }

    pub fn read(&self) -> u8 {
        match self.select_mode {
            SelectMode::Buttons => 0x20 | self.buttons,
            SelectMode::DirectionalPad => 0x10 | self.d_pad,
            SelectMode::Other => 0x3F,
        }
    }
}
