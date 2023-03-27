use log::debug;

use crate::{rom_parser::{RomParser, MirrorType}, rom, common::{CHR_Bank, PRG_Bank}};

pub struct Cartridge {
	// from iNES header
	num_prg_banks: u8,
	num_chr_banks: u8,
	mirror_type: MirrorType,
	has_battery: bool,
	has_trainer: bool,

	// cartridge ROM, RAM of PRG/CHR
	prg_rom: Vec<PRG_Bank>,
	chr_rom: Vec<CHR_Bank>
}

impl Cartridge {
	pub fn new(rom_parser: RomParser) -> Self {
		let num_prg_banks = rom_parser.header.prg_rom_size;
		let num_chr_banks = rom_parser.header.chr_rom_size;
		Cartridge {
			num_prg_banks,
			num_chr_banks,
			mirror_type: rom_parser.header.mirroring,
			has_battery: rom_parser.header.battery_prg_ram,
			has_trainer: rom_parser.header.trainer,
			prg_rom: rom_parser.prg_rom,
			chr_rom: rom_parser.chr_rom
		}
	}

	pub fn read_prg_rom(&self, num_bank: u8, addr: u16) -> u8 {
		let prg_bank = self.prg_rom.get(num_bank as usize).expect("The PRG bank number doesn't exist");
		prg_bank[addr as usize]
	}

	pub fn write_prg_rom(&mut self, num_bank: u8, addr: u16, value: u8) {
		let prg_bank = self.prg_rom.get_mut(num_bank as usize).expect("The CHR bank number doesn't exist");
		prg_bank[addr as usize] = value;
	}
}