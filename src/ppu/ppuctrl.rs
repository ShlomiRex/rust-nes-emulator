
pub struct PPUCtrl {
    pub register: u8
}

impl PPUCtrl {
    pub fn new() -> Self {
        Self { register: 0 }
    }
    
    pub fn nametable(&mut self) -> u8 {
        (self.register & 1) | (self.register & (1 << 1))
    }

    pub fn vram_addr_inc(&mut self) -> u8 {
        self.register & (1 << 2)
    }

    pub fn sprite_pattern_address(&mut self) -> u8 {
        self.register & (1 << 3)
    }

    pub fn bg_pattern_address(&mut self) -> u8 {
        self.register & (1 << 4)
    }

    pub fn sprite_size(&mut self) -> u8 {
        self.register & (1 << 5)
    }

    pub fn ppu_master_slave(&mut self) -> u8 {
        self.register & (1 << 6)
    }

    pub fn generate_NMI(&mut self) -> u8 {
        self.register & (1 << 7)
    }
}