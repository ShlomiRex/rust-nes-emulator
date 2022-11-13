extern crate hex;

use log::debug;

/// Addressable memory (64kb). Includes zero page, CPU ram, PPU registers, Cartidge memory, basically all available addressable memory.
pub struct MemoryBus {
	memory: Box<[u8; 65_536]>
}

pub struct ROM {
	pub rom: Box<[u8; 65_536]> 		// NOTE: ROM can be very big (8MB). For now I leave it at 64kb.
}

/// From page 11: https://www.nesdev.org/NESDoc.pdf
#[derive(Debug)]
enum MemoryMap {
	ZEROPAGE, 			// 0x0000 - 0x00FF
	STACK,				// 0x0100 - 0x01FF
	RAM, 				// 0x0200 - 0x07FF
	Mirrors0000_07FF, 	// 0x0800 - 0x1FFF
	MappedIO,			// 0x2000 - 0x4020
	ExpansionROM, 		// 0x4020 - 0x5FFF
	SRAM, 				// 0x6000 - 0x7FFF
	PrgRom,  			// 0x8000 - 0xFFFF
}

fn get_memory_map(addr: u16) -> MemoryMap {
	if addr <= 0x00FF {
		MemoryMap::ZEROPAGE
	} else if addr >= 0x0100 && addr < 0x0200 {
		MemoryMap::STACK
	} else if addr >= 0x0200 && addr < 0x0800 {
		MemoryMap::RAM
	} else if addr >= 0x0800 && addr < 0x2000{
		MemoryMap::Mirrors0000_07FF
	} else if addr >= 0x2000 && addr < 0x4020 {
		MemoryMap::MappedIO
	} else if addr >= 0x4020 && addr < 0x6000 {
		MemoryMap::ExpansionROM
	} else if addr >= 0x6000 && addr < 0x5FFF {
		MemoryMap::SRAM
	} else {
		MemoryMap::PrgRom
	}
}

impl MemoryBus {
	pub fn new() -> Self {
		MemoryBus { memory: Box::new([0; 65536]) }
	}

	fn debug_write(&self, addr: u16, data: u8) {
		let map = get_memory_map(addr);
		debug!("Writing to {:?}, address: {:#X}, data: {:#X}", map, addr, data);
	}

	fn debug_read(&self, addr: u16) {
		let map = get_memory_map(addr);
		debug!("Reading from {:?}, address: {:#X}", map, addr);
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
    fn test_ram() {
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