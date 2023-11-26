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
}
