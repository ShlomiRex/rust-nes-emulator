// Note:
// Address is 2 bytes, usize is 64 bits on x64 and 32 bits on x86.
// So converting u16 to usize never overflows, so casting is no problem.

pub struct RAM {
	ram: Box<[u8; 65_536]>
}

pub struct ROM {
	pub rom: Box<[u8; 65_536]> 		// NOTE: ROM can be very big. For now I leave it at 64kb.
}

impl RAM {
	pub fn new() -> Self {
		RAM { ram: Box::new([0; 65536]) }
	}

	pub fn write(&mut self, addr: u16, data: u8) {
		self.ram[addr as usize] = data;
	}

	pub fn read(&self, addr: u16) -> u8 {
		self.ram[addr as usize]
	}
}

impl ROM {
	// pub fn new() -> Self {
	// 	ROM { rom: Box::new([0; 65536]) }
	// }

	pub fn new(rom: Box<[u8; 65536]>) -> Self {
		ROM { rom }
	}
	
	pub fn read(&self, addr: u16) -> u8 {
		self.rom[addr as usize]
	}
}

#[cfg(test)]
mod tests {
	use super::*;

    #[test]
    fn ram_test() {
		let mut ram: RAM = RAM { ram: Box::new([0; 65536]) };
		let addr: u16 = 0x1234;
		ram.write(addr, 0xAB);
		ram.write(addr + 1, 0xCD);

		assert!(ram.read(0) == 0);
		assert!(ram.read(addr - 1) == 0x00);
		assert!(ram.read(addr    ) == 0xAB);
		assert!(ram.read(addr + 1) == 0xCD);
		assert!(ram.read(addr + 2) == 0x00);
    }
}