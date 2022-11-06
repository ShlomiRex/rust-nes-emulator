/// 2 bytes address
pub type Address = u16;

#[allow(non_snake_case)]
pub struct Bus {
	pub RAM: Box<[u8; 65_536]>
}

impl Bus {
	/// Write to RAM address
	pub fn write(&mut self, addr: Address, data: u8) {
		self.RAM[addr as usize] = data; //Address is 2 bytes, usize is 64 bits on x64 and 32 bits on x86, so it never overflows, so casting is no problem.
	}

	/// Read from RAM address
	pub fn read(&self, addr: Address) -> u8 {
		self.RAM[addr as usize] //Address is 2 bytes, usize is 64 bits on x64 and 32 bits on x86, so it never overflows, so casting is no problem.
	}
}