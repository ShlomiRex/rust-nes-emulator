use crate::{cpu::cpu::CPU, ppu::ppu::PPU, cartridge::Cartridge, rom_parser::RomParser};

pub struct NES {
	pub cpu: CPU
}

impl NES {
	fn new(cartridge: Cartridge) -> Self {	
		// Shared 32KB of lower memory, shared between CPU, PPU

		let ppu: PPU = PPU::new(&cartridge);
		
		// MMU is chip inside CPU.
		let cpu: CPU = CPU::new(cartridge, ppu);

		NES {
			cpu
		}
	}

	pub fn new_open_rom_file(path: &str) -> Self {
		let mut rom_parser = RomParser::new();
		rom_parser.parse(path);
	
		let cartridge: Cartridge = Cartridge::new_with_parser(rom_parser);
		NES::new(cartridge)
	}

	#[cfg(test)]
	pub fn new_custom_prg_rom(prg_rom: [u8;1024*32]) -> Self {
		let cartridge: Cartridge = Cartridge::new_with_custom_rom(prg_rom);
		NES::new(cartridge)
	}
}