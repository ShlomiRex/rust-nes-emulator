use log::debug;

use crate::{rom_parser::{RomParser, MirrorType}, common::{CHR_Bank, PRG_Bank}};

pub struct Cartridge {
	// from iNES header
	pub num_prg_banks: u8,
	num_chr_banks: u8,
	pub mapper_num: u8,
	mirror_type: MirrorType,
	has_battery: bool,
	has_trainer: bool,

	// cartridge ROM, RAM of PRG/CHR
	prg_rom: Vec<PRG_Bank>,
	chr_rom: Vec<CHR_Bank>
}

impl Cartridge {
	pub fn new_with_parser(rom_parser: RomParser) -> Self {
		Cartridge {
			num_prg_banks: rom_parser.header.prg_rom_size,
			num_chr_banks: rom_parser.header.chr_rom_size,
			mapper_num: rom_parser.header.mapper,
			mirror_type: rom_parser.header.mirroring,
			has_battery: rom_parser.header.battery_prg_ram,
			has_trainer: rom_parser.header.trainer,
			prg_rom: rom_parser.prg_rom,
			chr_rom: rom_parser.chr_rom
		}
	}

	pub fn new() -> Self {
		Cartridge {
			num_prg_banks: 2,
			num_chr_banks: 0,
			mapper_num: 0,
			mirror_type: MirrorType::HORIZONTAL,
			has_battery: false,
			has_trainer: false,
			prg_rom: vec![[0; 1024*16], [0; 1024*16]],
			chr_rom: vec![]
		}
	}

	pub fn new_with_custom_rom(rom: [u8;1024*32]) -> Self {
		let mut cartridge = Cartridge::new();

		// Copy first bank to cartridge
		let first_prg_bank = cartridge.prg_rom.get_mut(0).unwrap();
		first_prg_bank.copy_from_slice(&rom[0..1024*16]);

		// Copy second bank to cartridge
		let first_prg_bank = cartridge.prg_rom.get_mut(1).unwrap();
		first_prg_bank.copy_from_slice(&rom[1024*16..]);

		cartridge
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