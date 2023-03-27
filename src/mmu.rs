use log::debug;

use crate::cartridge::{Cartridge};

/// The MMU is located inside the CPU (real NES hardware). Its responsible to translate logical addresses to physical addresses.

pub struct MMU {
	// The physical cartridge inserted into the NES.
	cartridge: Cartridge,

	// The CPU can only access up to 2 program memory banks and 1 character bank at once. The MMU can switch between diffirent banks.
	activePRGBankNumberLower: u8,
	activePRGBankNumberUpper: u8,
	activeCHRBankNumber: u8
}

impl MMU {
	pub fn new(cartridge: Cartridge) -> Self {
		MMU {
			cartridge,
			activePRGBankNumberLower: 0,
			activePRGBankNumberUpper: 1,
			activeCHRBankNumber: 0
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
		debug!("Reading addr: {}, Location: {:?}", addr, mapped);

		if mapped == MemoryMap::PrgRom {
			let selected_prg_bank = 
				if addr >= 0xC000 {
					debug!("Reading upper PRG ROM");
					self.activePRGBankNumberUpper
				} else {
					debug!("Reading lower PRG ROM");
					self.activePRGBankNumberLower
				};
			let physical_addr = addr - 0xC000;
			self.cartridge.read_prg_rom(selected_prg_bank, physical_addr);
		}
		0
	}

	pub fn write_request(&self, addr: u16, value: u8) {

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