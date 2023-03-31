

pub struct PPU {
    // chr_rom: Vec<CHR_Bank>,
	// active_chr_rom_num: u8,
	// palette_table: [u8; 32],
	// vram: [u8; 2048],
	// oam_data: [u8; 256],
	// mirroring: MirrorType
	registers: [u8; 8],
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

	pub fn new() -> Self {
        PPU {
			registers: [0;8]
        }
    }

	pub fn read_register(&self, addr: u16) -> u8 {
		self.registers[addr as usize]
	}

	pub fn write_register(&mut self, addr: u16, value: u8) {
		self.registers[addr as usize] = value;
	}
}


