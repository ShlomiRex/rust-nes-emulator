use log::debug;

use crate::{cartridge::{Cartridge}};

/// The MMU is located inside the CPU (real NES hardware). Its responsible to translate logical addresses to physical addresses.

pub struct MMU {
	// Lower 32KB address space memory of CPU
	lower_memory: [u8; 1024*32],

	// The physical cartridge inserted into the NES.
	cartridge: Cartridge,

	// The CPU can only access up to 2 program memory banks and 1 character bank at once. The MMU can switch between diffirent banks.
	active_prgbank_number_lower: u8,
	active_prgbank_number_upper: u8,
	active_chrbank_number: u8
}

impl MMU {
	pub fn new(cartridge: Cartridge) -> Self {
		let lower_memory: [u8; 1024*32] = [0; 1024*32];

		// Defautl configuration: first bank goes to lower memory, second bank goes to upper memory
		let mut active_prgbank_number_lower = 0;
		let mut active_prgbank_number_upper = 1;

		// If there is only 1 bank, we MIRROR THE MEMORY for both lower 16KB and upper 16KB.
		if cartridge.num_prg_banks == 1 {
			active_prgbank_number_lower = 0;
			active_prgbank_number_upper = 0;
		}

		MMU {
			lower_memory,
			cartridge,
			active_prgbank_number_lower,
			active_prgbank_number_upper,
			active_chrbank_number: 0
		}
	}

	pub fn read_request(&self, addr: u16) -> u8 {
		let mapped = match addr {
			// Low 32KB
			0x0000..=0x00FF => MemoryMap::ZEROPAGE,
			0x0100..=0x01FF => MemoryMap::STACK,
			0x0200..=0x07FF => MemoryMap::RAM,
			0x0800..=0x1FFF => MemoryMap::Mirrors0000_07FF,
			0x2000..=0x401F => MemoryMap::MappedIO,
			0x4020..=0x5FFF => MemoryMap::ExpansionROM,
			0x6000..=0x7FFF => MemoryMap::SRAM,

			// High 32KB
			_ => MemoryMap::PrgRom
		};
		//debug!("Reading addr: 0x{:X}, Location: {:?}", addr, mapped);

		if mapped == MemoryMap::PrgRom {	
			if addr >= 0xC000 {
				//debug!("Reading upper PRG ROM");
				return self.cartridge.read_prg_rom(self.active_prgbank_number_upper, addr - 0xC000)
			} else {
				//debug!("Reading lower PRG ROM");
				return self.cartridge.read_prg_rom(self.active_prgbank_number_lower, addr - 0x8000)
			};
		} else if mapped == MemoryMap::MappedIO {
			if addr >= 0x2000 && addr <= 0x2007 {
				debug!("Reading from PPU register");

			}
		}
		return self.lower_memory[addr as usize]
	}

	pub fn write_request(&mut self, addr: u16, value: u8) {
		let mapped = match addr {
			// Low 32KB
			0x0000..=0x00FF => MemoryMap::ZEROPAGE,
			0x0100..=0x01FF => MemoryMap::STACK,
			0x0200..=0x07FF => MemoryMap::RAM,
			0x0800..=0x1FFF => MemoryMap::Mirrors0000_07FF,
			0x2000..=0x401F => MemoryMap::MappedIO,
			0x4020..=0x5FFF => MemoryMap::ExpansionROM,
			0x6000..=0x7FFF => MemoryMap::SRAM,

			// High 32KB
			_ => MemoryMap::PrgRom
		};
		//debug!("Writing to addr: 0x{:X}, Location: {:?}", addr, mapped);

		if mapped == MemoryMap::PrgRom {
			if addr >= 0xC000 {
				//debug!("Writing upper PRG ROM");
				return self.cartridge.write_prg_rom(self.active_prgbank_number_upper, addr - 0xC000, value)
			} else {
				//debug!("Writing lower PRG ROM");
				return self.cartridge.write_prg_rom(self.active_prgbank_number_lower, addr - 0x8000, value)
			};
		}

		self.lower_memory[addr as usize] = value;
	}
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