// Note:
// Address is 2 bytes, usize is 64 bits on x64 and 32 bits on x86.
// So converting u16 to usize never overflows, so casting is no problem.

// Address space: 		0x0000 - 0xFFFF (64k)
// Zero page: 			0x0000 - 0x00FF (256 bytes)
// Stack: 				0x0100 - 0x01FF (after zero page, 256 bytes)

extern crate hex;

use log::debug;

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

	fn debug_write(&self, addr: u16, data: u8) {
		if addr <= 0xFF {
			debug!("Writing to zero page, address: {:#X}, data: {:#X}", addr, data);
		} else if addr >= 0x100 && addr <= 0x1FF {
			debug!("Writing to stack, address: {:#X}, data: {:#X}", addr, data);
		} else {
			debug!("Writing to higher up, address: {:#X}, data: {:#X}", addr, data);
		}
	}

	fn debug_read(&self, addr: u16) {
		if addr <= 0xFF {
			debug!("Reading from zero page, address: {:#X}", addr);
		} else if addr >= 0x100 && addr <= 0x1FF {
			debug!("Reading from stack, address: {:#X}", addr);
		} else {
			debug!("Reading from higher up, address: {:#X}", addr);
		}
	}
	
	/// Write a single byte to memory.
	pub fn write(&mut self, addr: u16, data: u8) {
		self.debug_write(addr, data);
		self.ram[addr as usize] = data;
	}

	/// Read a single byte from memory.
	pub fn read(&self, addr: u16) -> u8 {
		self.debug_read(addr);
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
	
	//TODO: Maybe convert to inline? This function can be called millions of times a second!
	pub fn read(&self, addr: u16) -> u8 {
		self.rom[addr as usize]
	}
}

/// Write to array the bytes from string, represented by hex with spaces.
pub fn write_rom(rom_memory: &mut [u8;65_536], dump: &str) {
	let split = dump.split(" ");
	let mut i = 0;
	for s in split {
		let z = hex::decode(s).unwrap();
		rom_memory[i] = z[0];
		i += 1;
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