pub struct RAM {
	ram: Box<[u8; 65_536]>
}

impl RAM {
	pub fn new() -> RAM {
		RAM { ram: Box::new([0; 65536]) }
	}

	/// Write to RAM address
	pub fn write(&mut self, addr: u16, data: u8) {
		self.ram[addr as usize] = data; //Address is 2 bytes, usize is 64 bits on x64 and 32 bits on x86, so it never overflows, so casting is no problem.
	}

	/// Read from RAM address
	pub fn read(&self, addr: u16) -> u8 {
		self.ram[addr as usize] //Address is 2 bytes, usize is 64 bits on x64 and 32 bits on x86, so it never overflows, so casting is no problem.
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