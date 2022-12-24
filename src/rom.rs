pub struct ROM {
	pub rom: Vec<u8> 		// ROM size is not fixed; However, its usually 32kb.
}

impl ROM {
	pub fn read(&self, addr: u16) -> u8 {
		self.rom[addr as usize]
	}
}
