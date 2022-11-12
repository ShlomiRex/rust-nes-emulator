
pub struct PPUStatus {
    pub register: u8
}

impl PPUStatus {
    pub fn new() -> Self {
        Self { register: 0 }
    }
    
    pub fn sprite_overflow(&mut self) -> u8 {
        self.register & (1 << 5)
    }

    pub fn sprite_0_hit(&mut self) -> u8 {
        self.register & (1 << 6)
    }

    pub fn vertical_blank_started(&mut self) -> u8 {
        self.register & (1 << 7)
    }
}