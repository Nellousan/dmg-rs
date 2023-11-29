use std::{fs, io, result};

use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not read cartridge: {0}.")]
    Loading(#[from] io::Error),
    #[error("Cartridge size is inferior to 0x8000.")]
    InvalidRomSize,
    #[error("Invalid Header: {0}")]
    InvalidHeader(String),
    #[error("Unimplemented MBC: {0}")]
    UnimplementedMBC(u8),
}

pub type Result<T> = result::Result<T, Error>;

pub trait Cartridge: Send {
    fn write_8(&mut self, address: u16, value: u8);
    fn write_16(&mut self, address: u16, value: u16);
    fn read_8(&self, address: u16) -> u8;
    fn read_16(&self, address: u16) -> u16;
}

pub fn from_file(path: &str) -> Result<Box<dyn Cartridge>> {
    let rom = fs::read(path).map_err(|err| Error::Loading(err))?;

    if rom.len() < 0x8000 {
        return Err(Error::InvalidRomSize);
    }

    let mbc = rom[0x0147];
    match mbc {
        0x00 => Ok(Box::new(CartridgeROM::new(rom)?)),
        0x01..=0x03 => Ok(Box::new(CartridgeMBC1::new(rom)?)),
        _ => Err(Error::UnimplementedMBC(mbc)),
    }
}

// Test roms

pub fn test_rom_from_file(path: &str) -> Result<Box<dyn Cartridge>> {
    let rom = fs::read(path).map_err(|err| Error::Loading(err))?;

    Ok(Box::new(CartridgeROM { rom, _rom_size: 0 }))
}

// ROM Cartridge

pub struct CartridgeROM {
    rom: Vec<u8>,
    _rom_size: u8,
}

impl CartridgeROM {
    fn new(rom: Vec<u8>) -> Result<Self> {
        let _rom_size = rom[0x0148];

        Ok(Self { rom, _rom_size })
    }
}

impl Cartridge for CartridgeROM {
    fn write_8(&mut self, _address: u16, _value: u8) {
        error!("Tried to write to ROM Cartridge !");
    }

    fn write_16(&mut self, _address: u16, _value: u16) {
        error!("Tried to write to ROM Cartridge !");
    }

    fn read_8(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }

    fn read_16(&self, address: u16) -> u16 {
        let n1 = self.read_8(address);
        let n2 = self.read_8(address + 1);
        u16::from_le_bytes([n1, n2])
    }
}

// MBC1 Cartridge

#[derive(Default)]
pub struct CartridgeMBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    _rom_bank_count: u32,
    _ram_bank_count: u32,
    selected_rom_bank: u32,
    selected_ram_bank: u32,
}

impl CartridgeMBC1 {
    pub fn new(rom: Vec<u8>) -> Result<Self> {
        let rom_size = rom[0x0148];
        let _rom_bank_count = 1 << (rom_size + 1);

        let ram_size = rom[0x0149];
        let (ram, _ram_bank_count) = match ram_size {
            0x00 => (vec![0u8; 0], 0),
            0x02 => (vec![0u8; 0x2000], 1),
            0x03 => (vec![0u8; 0x4000], 4),
            0x04 => (vec![0u8; 0x20000], 16),
            0x05 => (vec![0u8; 0x10000], 8),
            _ => {
                return Err(Error::InvalidHeader("Invalid RAM size header.".into()));
            }
        };
        Ok(Self {
            rom,
            ram,
            _rom_bank_count,
            _ram_bank_count,
            selected_rom_bank: 1,
            ..Default::default()
        })
    }

    fn select_rom_bank(&mut self, value: u8) {
        let mut value = value & 0x1F;
        if value == 0 {
            value = 1;
        }
        self.selected_rom_bank = value as u32;
    }

    fn select_ram_bank(&mut self, value: u8) {
        let value = value & 0x03;

        self.selected_ram_bank = value as u32;
    }

    fn rom_read_8(&self, address: u16) -> u8 {
        self.rom[self.selected_rom_bank as usize * 0x4000 + address as usize - 0x4000]
    }

    fn rom_read_16(&self, address: u16) -> u16 {
        let n1 = self.read_8(address);
        let n2 = self.read_8(address + 1);
        u16::from_le_bytes([n1, n2])
    }

    fn ram_write_8(&mut self, address: u16, value: u8) {
        self.ram[self.selected_ram_bank as usize * 0x2000 + address as usize - 0xA000] = value;
    }

    fn ram_write_16(&mut self, address: u16, value: u16) {
        let bytes = value.to_le_bytes();

        self.ram[self.selected_ram_bank as usize * 0x2000 + address as usize - 0xA000] = bytes[0];
        self.ram[self.selected_ram_bank as usize * 0x2000 + (address + 1) as usize - 0xA000] =
            bytes[1];
    }

    fn ram_read_8(&self, address: u16) -> u8 {
        self.ram[self.selected_ram_bank as usize * 0x2000 + address as usize - 0xA000]
    }

    fn ram_read_16(&self, address: u16) -> u16 {
        let n1 = self.read_8(address);
        let n2 = self.read_8(address + 1);
        u16::from_le_bytes([n1, n2])
    }
}

impl Cartridge for CartridgeMBC1 {
    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => (),
            0x2000..=0x3FFF => self.select_rom_bank(value),
            0x4000..=0x5FFF => self.select_ram_bank(value),
            0x6000..=0x7FFF => unimplemented!(),
            0xA000..=0xBFFF => self.ram_write_8(address, value),
            _ => unreachable!(),
        }
    }

    fn write_16(&mut self, address: u16, value: u16) {
        match address {
            0xA000..=0xBFFF => self.ram_write_16(address, value),
            _ => unreachable!(),
        }
    }

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.rom[address as usize],
            0x4000..=0x7FFF => self.rom_read_8(address),
            0xA000..=0xBFFF => self.ram_read_8(address),
            _ => unreachable!(),
        }
    }

    fn read_16(&self, address: u16) -> u16 {
        match address {
            0x0000..=0x7FFF => self.rom_read_16(address),
            0xA000..=0xBFFF => self.ram_read_16(address),
            _ => unreachable!(),
        }
    }
}
