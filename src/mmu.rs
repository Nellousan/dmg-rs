use crate::cartridge::Cartridge;

pub struct MemoryMapUnit {
    memory: Vec<u8>,
    cartridge: Box<dyn Cartridge>,
}

impl MemoryMapUnit {
    pub fn new(cartridge: Box<dyn Cartridge>) -> Self {
        MemoryMapUnit {
            memory: vec![0u8; 0xFFFF],
            cartridge,
        }
    }

    pub fn read_8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cartridge.read_8(address),
            _ => self.memory[address as usize],
        }
    }

    pub fn read_16(&self, address: u16) -> u16 {
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cartridge.read_16(address),
            _ => {
                let n1 = self.read_8(address);
                let n2 = self.read_8(address + 1);
                u16::from_le_bytes([n1, n2])
            }
        }
    }

    pub fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cartridge.write_8(address, value),
            _ => self.memory[address as usize] = value,
        }
    }

    pub fn write_16(&mut self, address: u16, value: u16) {
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cartridge.write_16(address, value),
            _ => {
                let bytes = value.to_le_bytes();
                self.memory[address as usize] = bytes[0];
                self.memory[(address + 1) as usize] = bytes[0];
            }
        }
    }
}
