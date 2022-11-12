#[derive(Default)]
pub struct Register<T> {
    pub data: T,
}

impl Register<u8> {
    pub fn new() -> Self {
        Self { data: 0 }
    }

    pub fn is_set(self, i: usize) -> bool {
        if i < 0 || i > 8 {
            panic!("Out of bounds error");
        }
        return (self.data & (1 << i)) == 1
    }

    pub fn set(&mut self, i: usize) -> () {
        if i < 0 || i > 8 {
            panic!("Out of bounds error");
        }
        self.data |= 1 << i
    }

    pub fn unset(&mut self, i: usize) -> () {
        if i < 0 || i > 8 {
            panic!("Out of bounds error");
        }
        self.data ^= 1 << i
    }

    pub fn flip(&mut self, i: usize) -> () {
        if i < 0 || i > 8 {
            panic!("Out of bounds error");
        }
        self.data ^= 1 << i
    }

    pub fn get_mask(self, m: u8) -> u8 {
        return self.data & m
    }

    pub fn set_mask(&mut self, m: u8) -> () {
        self.data &= m
    }

    pub fn zero(&mut self) -> () {
        self.data ^= 0
    }
}

impl Register<u16> {
    pub fn new() -> Self {
        Self { data: 0 }
    }

    pub fn is_set(self, i: usize) -> bool {
        if i < 0 || i > 16 {
            panic!("Out of bounds error");
        }
        return (self.data & (1 << i)) == 1
    }

    pub fn set(&mut self, i: usize) -> () {
        if i < 0 || i > 16 {
            panic!("Out of bounds error");
        }
        self.data |= 1 << i
    }

    pub fn unset(&mut self, i: usize) -> () {
        if i < 0 || i > 16 {
            panic!("Out of bounds error");
        }
        self.data ^= 1 << i
    }

    pub fn flip(&mut self, i: usize) -> () {
        if i < 0 || i > 16 {
            panic!("Out of bounds error");
        }
        self.data ^= 1 << i
    }

    pub fn get_mask(self, m: u16) -> u16 {
        return self.data & m
    }

    pub fn set_mask(&mut self, m: u16) -> () {
        self.data &= m
    }

    pub fn zero(&mut self) -> () {
        self.data ^= 0
    }
}