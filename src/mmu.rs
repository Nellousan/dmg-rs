use crate::cartridge::Cartridge;

pub struct MMU {
    memory: Vec<u8>,
    cartridge: Box<dyn Cartridge>,
}

impl MMU {
    pub fn new(cartridge: Box<dyn Cartridge>) -> Self {
        MMU {
            memory: vec![0u8; 0xFFFF],
            cartridge,
        }
    }
}
