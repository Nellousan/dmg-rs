use std::sync::Arc;

use crate::cartridge::Cartridge;

#[derive(Debug)]
pub struct MemoryMapUnit {
    memory: [u8; 0x10000],
    cartridge: Box<dyn Cartridge>,
    boot_rom: &'static [u8; 256],
}

impl MemoryMapUnit {
    pub fn new(cartridge: Box<dyn Cartridge>) -> Self {
        MemoryMapUnit {
            memory: [0u8; 0x10000],
            cartridge,
            boot_rom: include_bytes!("../dmg_boot.bin"),
        }
    }

    fn boot_rom_enabled(&self) -> bool {
        self.memory[0xFF50] == 0
    }

    pub fn read_8(&self, address: u16) -> u8 {
        if self.boot_rom_enabled() && address <= 0xFF {
            return self.boot_rom[address as usize];
        }

        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cartridge.read_8(address),
            0xFF00 => 0x0F, // TEMPORARY UNTIL INPUTS ARE IMPLEMENTED
            _ => self.memory[address as usize],
        }
    }

    pub fn read_16(&self, address: u16) -> u16 {
        if self.boot_rom_enabled() && address <= 0xFF {
            let n1 = self.read_8(address);
            let n2 = self.read_8(address + 1);
            return u16::from_le_bytes([n1, n2]);
        }

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
            0xFF46 => self.dma_transfer(value),
            _ => self.memory[address as usize] = value,
        }
    }

    pub fn write_16(&mut self, address: u16, value: u16) {
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cartridge.write_16(address, value),
            _ => {
                let bytes = value.to_le_bytes();
                self.memory[address as usize] = bytes[0];
                self.memory[(address + 1) as usize] = bytes[1];
            }
        }
    }

    pub fn get_memory_dump(&self) -> Arc<[u8; 0x10000]> {
        let mut memory = self.memory.clone();
        let rom = self.cartridge.dump_rom();

        memory.as_mut_slice()[0x0000..0x8000].copy_from_slice(&rom);
        if self.boot_rom_enabled() {
            memory.as_mut_slice()[0..256].copy_from_slice(self.boot_rom);
        }
        Arc::new(memory)
    }

    pub fn vram(&self) -> Vec<u8> {
        self.memory[0x8000..=0x9FFF].to_vec()
    }

    fn dma_transfer(&mut self, source: u8) {
        let starting_address = (source as u16) << 8;

        for i in 0..0x100 {
            let address = starting_address + i;
            let destination = 0xFE00 + i;
            let value = self.read_8(address);
            self.write_8(destination, value);
        }
    }

    pub fn borrow_rom(&self) -> &[u8] {
        &self.cartridge.borrow_rom()
    }
}
