use crate::rom::ROM;
use crate::memory::Memory;
use log::debug;

pub trait Mapping {
	fn read(&self, addr: u16) -> u8;
	fn write(&mut self, addr: u16, data: u8);
}

/// From page 11: https://www.nesdev.org/NESDoc.pdf
#[derive(Debug, PartialEq)]
enum MemoryMap {
	ZEROPAGE, 			// 0x0000 - 0x00FF
	STACK,				// 0x0100 - 0x01FF
	RAM, 				// 0x0200 - 0x07FF
	Mirrors0000_07FF, 	// 0x0800 - 0x1FFF
	MappedIO,			// 0x2000 - 0x401F
	ExpansionROM, 		// 0x4020 - 0x5FFF
	SRAM, 				// 0x6000 - 0x7FFF
	PrgRom,  			// 0x8000 - 0xFFFF
}

fn get_memory_map(addr: u16) -> MemoryMap {
	match addr {
		0x0000..=0x00FF => MemoryMap::ZEROPAGE,
		0x0100..=0x01FF => MemoryMap::STACK,
		0x0200..=0x07FF => MemoryMap::RAM,
		0x0800..=0x1FFF => MemoryMap::Mirrors0000_07FF,
		0x2000..=0x401F => MemoryMap::MappedIO,
		0x4020..=0x5FFF => MemoryMap::ExpansionROM,
		0x6000..=0x7FFF => MemoryMap::SRAM,
		_ => MemoryMap::PrgRom
	}
}

pub struct Mapper0 {
	memory: [u8; 32_768],
	rom: ROM,
	rom_start: u16 			// ROM can be 16kb, which means, we need to align it to the last bytes of addressable memory.
}

impl Mapper0 {
	pub fn new(mut memory: Memory, rom: ROM) -> Self {
		let mut rom_start = 0x8000;
		if rom.rom.len() == 1024 * 16 {
			rom_start = 0x8000 + 0x4000;
		}
		Mapper0{
			memory,
			rom,
			rom_start
		}
	}
}

impl Mapping for Mapper0 {
	fn read(&self, addr: u16) -> u8 {
		if addr < 0x8000 {
			let map = get_memory_map(addr);
			debug!("Reading from {:?}, address: {:#X}", map, addr);
			self.memory[addr as usize]
		} else {
			self.rom.read(addr - self.rom_start)
		}
	}
	fn write(&mut self, addr: u16, data: u8) {
		let map = get_memory_map(addr);
		if addr < 0x8000 {
			debug!("Writing to {:?}, address: {:#X}, data: {:#X}", map, addr, data);
			self.memory[addr as usize] = data;
		} else {
			panic!("Cannot write to memory location: {:#X}, its read only! Memory region: {:?}", addr, map);
		}
	}
}

