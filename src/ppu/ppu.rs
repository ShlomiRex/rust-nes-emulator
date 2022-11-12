use super::registers::Registers;

pub struct PPU {
    pub registers: Registers
}

impl PPU {
    pub fn new() -> Self {
        let mut ppu = PPU {
            registers: Registers::new(),
        };
        ppu
    }
}