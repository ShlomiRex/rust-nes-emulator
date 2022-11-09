// Note:
// Address is 2 bytes, usize is 64 bits on x64 and 32 bits on x86.
// So converting u16 to usize never overflows, so casting is no problem.

pub struct RAM {
	ram: Box<[u8; 65_536]>
}

pub struct ROM {
	pub rom: Box<[u8; 65_536]> 		// NOTE: ROM can be very big (8MB). For now I leave it at 64kb.
}

impl RAM {
	pub fn new() -> Self {
		RAM { ram: Box::new([0; 65536]) }
	}

	/// Write a single byte to memory.
	pub fn write(&mut self, addr: u16, data: u8) {
		self.ram[addr as usize] = data;
	}

	/// Read a single byte from memory.
	pub fn read(&self, addr: u16) -> u8 {
		self.ram[addr as usize]
	}

	/// Read adress from memory, which is 2 bytes long (addr, addr+1). Takes into account little-endian order to combine LSB with MSB.
	pub fn read_address(&self, addr: u16) -> u16 {
		let lsb = self.ram[addr as usize] 		as u16;
		let msb = self.ram[addr as usize + 1] 	as u16;
		(msb << 4) | lsb
	}
}

impl ROM {
	// pub fn new() -> Self {
	// 	ROM { rom: Box::new([0; 65536]) }
	// }

	pub fn new(rom: Box<[u8; 65536]>) -> Self {
		ROM { rom }
	}
	
	//TODO: Maybe convert to inline? This function can be called millions of times a second!
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