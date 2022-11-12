
pub struct PPUMask {
    pub register: u8
}

impl PPUMask {
    pub fn new() -> Self {
        Self { register: 0 }
    }
    
    pub fn greyscale(&mut self) -> u8 {
        self.register & 1
    }

    pub fn show_bg_leftmost_8(&mut self) -> u8 {
        self.register & (1 << 1)
    }

    pub fn show_sprites_leftmost_8(&mut self) -> u8 {
        self.register & (1 << 2)
    }

    pub fn show_bg(&mut self) -> u8 {
        self.register & (1 << 3)
    }

    pub fn show_sprites(&mut self) -> u8 {
        self.register & (1 << 4)
    }

    pub fn emphasize_red(&mut self) -> u8 {
        self.register & (1 << 5)
    }

    pub fn emphasize_green(&mut self) -> u8 {
        self.register & (1 << 6)
    }

    pub fn emphasize_blue(&mut self) -> u8 {
        self.register & (1 << 7)
    }
}