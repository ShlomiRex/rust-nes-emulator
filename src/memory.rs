// Note:
// Address is 2 bytes, usize is 64 bits on x64 and 32 bits on x86.
// So converting u16 to usize never overflows, so casting is no problem.


// https://web.archive.org/web/20210803073202/http://www.obelisk.me.uk/6502/architecture.html
// Zero page: 0x0000 - 0x00FF : is the focus of a number of special addressing modes that result in shorter (and quicker) instructions or allow indirect access to the memory (256 bytes of memory)
// Stack: 	  0x0100 - 0x01FF : is reserved for the system stack and which cannot be relocated. (256 bytes of stack!)
// Reserved memory: 0xFFFA - 0xFFFF (last 6 bytes) : must be programmed with the addresses of the non-maskable interrupt handler ($FFFA/B), the power on reset location ($FFFC/D) and the BRK/interrupt request handler ($FFFE/F) respectively.


extern crate hex;

use log::debug;

/// Addressable memory (64kb). Includes zero page, CPU ram, PPU registers, Cartidge memory, basically all available addressable memory.
pub struct MemoryBus {
	memory: Box<[u8; 65_536]>
}

pub struct ROM {
	pub rom: Box<[u8; 65_536]> 		// NOTE: ROM can be very big (8MB). For now I leave it at 64kb.
}

enum MemoryMap {
	ZEROPAGE, 			// 0x0000 - 0x00FF
	STACK,				// 0x0100 - 0x01FF
	MappedIO,			// 0x2000 - 0x6000
	PpuMask, 			// 0x2001
	PpuStatus, 			// 0x2002
	// InterruptVectors, 	// 0xFFFD, 0xFFFE, 0xFFFF
	OTHER,  			// everything else (it will be completed when I understand memory better)
}

/// Read = if you intend to read or write to mm.
fn get_memory_map(addr: u16, read: bool) -> MemoryMap {
	if addr <= 0x00FF {
		MemoryMap::ZEROPAGE
	} else if addr >= 0x100 && addr < 0x200 {
		MemoryMap::STACK
	} else if addr >= 0x2000 && addr < 0x6000 {
		if addr == 0x2002 {
			if read {
				MemoryMap::PpuStatus
			} else {
				panic!("You can't write to PPU Status memory. It's read only.");
			}
		} else {
			MemoryMap::MappedIO
		}
	} else {
		MemoryMap::OTHER
	}
}

impl MemoryBus {
	pub fn new() -> Self {
		MemoryBus { memory: Box::new([0; 65536]) }
	}

	fn debug_write(&self, addr: u16, data: u8) {
		let map = get_memory_map(addr, false);
		match map {
			MemoryMap::ZEROPAGE 		=> debug!("Writing to zero page, address: {:#X}, data: {:#X}", addr, data),
			MemoryMap::STACK 			=> debug!("Writing to stack, address: {:#X}, data: {:#X}", addr, data),
			MemoryMap::MappedIO			=> debug!("Writing to memory mapped i/o, address: {:#X}, data: {:#X}", addr, data),
			MemoryMap::PpuStatus 		=> (),
			MemoryMap::PpuMask 			=> debug!("Writing to PPU mask, address: {:#X}, data: {:#X}", addr, data),
			MemoryMap::OTHER 			=> debug!("Writing to address: {:#X}, data: {:#X}", addr, data)
		}
	}

	fn debug_read(&self, addr: u16) {
		let map = get_memory_map(addr, true);
		match map {
			MemoryMap::ZEROPAGE 		=> debug!("Reading from zero page, address: {:#X}", addr),
			MemoryMap::STACK 			=> debug!("Reading from stack, address: {:#X}", addr),
			MemoryMap::MappedIO			=> debug!("Reading from memory mapped i/o, address: {:#X}", addr),
			MemoryMap::PpuStatus 		=> debug!("Reading from PPU status, address: {:#X}", addr),
			MemoryMap::PpuMask 			=> debug!("Reading from PPU mask, address: {:#X}", addr),
			MemoryMap::OTHER 			=> debug!("Reading from	address: {:#X}", addr)
		}
	}
	
	/// Write a single byte to memory.
	pub fn write(&mut self, addr: u16, data: u8) {
		self.debug_write(addr, data);
		self.memory[addr as usize] = data;
	}

	/// Read a single byte from memory.
	pub fn read(&self, addr: u16) -> u8 {
		self.debug_read(addr);
		self.memory[addr as usize]
	}
}

impl ROM {
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
		let mut ram: MemoryBus = MemoryBus { memory: Box::new([0; 65536]) };
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