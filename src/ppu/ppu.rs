use crate::{common::{self, bits, CHR_Bank}, cartridge::Cartridge};



pub struct PPU {
	// active_chr_rom_num: u8,
	// oam_data: [u8; 256],
	// mirroring: MirrorType
	registers: [u8; 8],
	chr_rom: [u8;1024*8],					// 	address space: 0x0000-0x1FFF
	name_table: [u8; 2048], 		// vram		address space: 0x2000-0x3EFF
	palette_table: [u8; 32],				// 	address space: 0x3F00-0x3FFF
}

/*
Control Register 1 (PPUCTRL) - 		CPU Address: 0x2000
Control Register 2 (PPUMASK) - 		CPU Address: 0x2001
Status Register (PPUSTATUS) - 		CPU Address: 0x2002
OAM Address Register (OAMADDR) - 	CPU Address: 0x2003
OAM Data Register (OAMDATA) - 		CPU Address: 0x2004
Scroll Register (PPUSCROLL) - 		CPU Address: 0x2005
Address Register (PPUADDR) - 		CPU Address: 0x2006
Data Register (PPUDATA) - 			CPU Address: 0x2007

OAM DMA Register (DMA) - 			CPU Address: 0x4014
*/

impl PPU {
	// pub fn new(mm_ppu_registers: &'a [u8], cartridge: &'a Cartridge) -> Self {
    //     PPU {
	// 		registers: mm_ppu_registers,
	// 		cartridge
    //     }
    // }

	pub fn new(cartridge: &Cartridge) -> Self {
		// CHR ROM must have at least 1 bank, there can't be 0 CHR ROM data. In case of testing, we fill zeros.
		let mut chr_rom: [u8;1024*8] = [0;1024*8];

		//TODO: We need to handle more than 1 CHR ROM bank. For now I just want NES program to work.
		let first_chr_rom_bank = cartridge.chr_rom.get(0);
	
		if first_chr_rom_bank.is_some() {
			// Copy CHR ROM data from cartridge to local scope, and now PPU will own this cloned data.
			chr_rom.copy_from_slice(&first_chr_rom_bank.unwrap()[0..1024*8]);
		}

		//TODO: Init name_table and palette table
        PPU {
			registers: [0;8],
			chr_rom,
			name_table: [0;2048],
			palette_table: [0;32]
        }
    }

	pub fn read_register(&mut self, addr: u16) -> u8 {
		let result = self.registers[addr as usize];

		// clear bit 7
		let mut cleared_bit_7 = result;
		bits::set(&mut cleared_bit_7, 7, false);
		self.registers[addr as usize] = cleared_bit_7;

		result
	}

	pub fn write_register(&mut self, addr: u16, value: u8) {
		self.registers[addr as usize] = value;
	}
}


