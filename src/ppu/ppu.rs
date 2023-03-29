use super::registers::Registers;

pub struct PPU {
    pub registers: Registers
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            registers: Registers::new(),
        }
    }
}