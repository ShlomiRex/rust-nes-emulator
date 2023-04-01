use log::debug;

use crate::{cartridge::{Cartridge}, ppu::ppu::PPU, cpu::cpu::LowerMemory, apu::apu::APU};

/// The MMU is located inside the CPU (real NES hardware). Its responsible to translate logical addresses to physical addresses.
/// I intend to use MMU as bus to other components, such as PPU and cartridge.

pub struct MMU {
	// Lower 32KB address space memory of CPU
	//TODO: This should not be here. We store the exact memory we need elsewhere.
	lower_memory: [u8; 1024*32],		

	// The CPU can only access up to 2 program memory banks and 1 character bank at once. The MMU can switch between diffirent banks.
	active_prgbank_number_lower: u8,
	active_prgbank_number_upper: u8,
	active_chrbank_number: u8
}

impl MMU {
	pub fn new(lower_memory: [u8; 1024*32], cartridge: &Cartridge) -> Self {
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
			active_prgbank_number_lower,
			active_prgbank_number_upper,
			active_chrbank_number: 0
		}
	}

	pub fn read_request(&self, cartridge: &Cartridge, ppu: &mut PPU, addr: u16, lower_memory: &LowerMemory) -> u8 {
		match addr {
			// High 32KB
			0x8000..=0xBFFF => {
				// Lower PRG ROM
				cartridge.read_prg_rom(self.active_prgbank_number_lower, addr - 0x8000)
			}

			0xC000..=0xFFFF => {
				// Upper PRG ROM
				cartridge.read_prg_rom(self.active_prgbank_number_upper, addr - 0xC000)
			}

			// Low 32KB
			0x0000..=0x00FF => {
				// Zero page
				lower_memory.zero_page[addr as usize]
			}
			0x2000..=0x2007 => {
				// PPU registers
				ppu.read_register(addr - 0x2000)
			}
			0x2008..=0x3FFF => {
				// Mirrored PPU registers
				todo!();
				ppu.read_register((addr - 0x2000) % 8)
			}

			_ => {
				println!("addr: 0x{:X}", addr);
				todo!();
				//TODO: Lower memory should not contain values in range 0x2000-0x2008 and more, instead, you should have seperate memory for zeropage, stack, RAM, and more.
				self.lower_memory[addr as usize]
			}
		}
	}

	pub fn write_request(&mut self, ppu: &mut PPU, addr: u16, value: u8, lower_memory: &mut LowerMemory, apu: &mut APU) {
		match addr {
			// High 32KB
			0x8000..=0xBFFF => {
				//debug!("Writing lower PRG ROM");
				//self.cartridge.write_prg_rom(self.active_prgbank_number_lower, addr - 0x8000, value)

				//TODO: We should never write to ROM
				todo!();
			}
			0xC000..=0xFFFF => {
				//debug!("Writing upper PRG ROM");
				//self.cartridge.write_prg_rom(self.active_prgbank_number_upper, addr - 0xC000, value)

				//TODO: We should never write to ROM
				todo!();
			}

			// Low 32KB
			0x0000..=0x00FF => {
				// Zero page
				lower_memory.zero_page[addr as usize] = value
			}
			0x2000..=0x2007 => {
				// PPU registers
				ppu.write_register(addr - 0x2000, value);
			}
			0x2008..=0x3FFF => {
				// Mirrored PPU registers
				todo!();
				ppu.write_register((addr - 0x2000) % 8, value);
			}
			0x4000..=0x4017 => {
				// APU registers
				println!("addr: 0x{:X}", addr);
				apu.registers[(addr - 0x4000) as usize] = value;
			}

			_ => {
				//TODO: Lower memory should not contain values in range 0x2000-0x2008 and more, instead, you should have seperate memory for zeropage, stack, RAM, and more.
				println!("addr: 0x{:X}", addr);
				todo!();
				
				self.lower_memory[addr as usize] = value;
			}
		}
	}
}