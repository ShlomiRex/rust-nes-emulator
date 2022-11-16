extern crate hex;

use log::debug;

pub type Memory = [u8; 32_768];

/// Addressable memory (64kb). Includes zero page, CPU ram, PPU registers, Cartidge memory, basically all available addressable memory.
pub struct MemoryBus {
	memory: Memory,
	rom: ROM
}

pub struct ROM {
	pub rom: Vec<u8> 		// ROM size is not fixed; However, its usually 32kb.
}

/// From page 11: https://www.nesdev.org/NESDoc.pdf
#[derive(Debug, PartialEq)]
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
	pub fn new(memory: Memory, rom: ROM) -> Self {
		MemoryBus {
			memory,
			rom
		}
	}

	fn debug_write(&self, addr: u16, data: u8) {
		let map = get_memory_map(addr);
		debug!("Writing to {:?}, address: {:#X}, data: {:#X}", map, addr, data);
	}

	fn debug_read(&self, addr: u16) {
		let map = get_memory_map(addr);
		if map != MemoryMap::PrgRom {
			debug!("Reading from {:?}, address: {:#X}", map, addr);
		}
	}
	
	/// Write a single byte to memory.
	pub fn write(&mut self, addr: u16, data: u8) {
		self.debug_write(addr, data);
		if addr < 0x8000 {
			self.memory[addr as usize] = data;
		} else {
			panic!("Cannot write to memory location: {}, its read only!", addr);
		}
	}

	/// Read a single byte from memory.
	pub fn read(&self, addr: u16) -> u8 {
		self.debug_read(addr);
		if addr < 0x8000 {
			self.memory[addr as usize]
		} else {
			self.rom.read(addr - 0x8000)
		}
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
