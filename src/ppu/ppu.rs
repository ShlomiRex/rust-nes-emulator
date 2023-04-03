use crate::{common::{self, bits, CHR_Bank}, cartridge::Cartridge};



pub struct PPU {
	// active_chr_rom_num: u8,
	// oam_data: [u8; 256],
	// mirroring: MirrorType
	registers: [u8; 8],
	chr_rom: [u8;1024*8],					// 	address space: 0x0000-0x1FFF
	name_table: [u8; 2048], 		// vram		address space: 0x2000-0x3EFF
	palette_table: [u8; 32],				// 	address space: 0x3F00-0x3FFF (Background palette: 0x3F00-0x3F10 and Sprite palette: 0x3F10-0x3FFF)


	pub ppu_status: u8
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

		let system_palette: [(u8, u8, u8); 64] = [
			(0x52, 0x52, 0x52), /* 0x00 */
			(0x01, 0x1a, 0x51), /* 0x01 */
			(0x0f, 0x0f, 0x65), /* 0x02 */
			(0x23, 0x06, 0x63), /* 0x03 */
			(0x36, 0x03, 0x4b), /* 0x04 */
			(0x40, 0x04, 0x26), /* 0x05 */
			(0x3f, 0x09, 0x04), /* 0x06 */
			(0x32, 0x13, 0x00), /* 0x07 */
			(0x1f, 0x20, 0x00), /* 0x08 */
			(0x0b, 0x2a, 0x00), /* 0x09 */
			(0x00, 0x2f, 0x00), /* 0x0a */
			(0x00, 0x2e, 0x0a), /* 0x0b */
			(0x00, 0x26, 0x2d), /* 0x0c */
			(0x00, 0x00, 0x00), /* 0x0d */
			(0x00, 0x00, 0x00), /* 0x0e */
			(0x00, 0x00, 0x00), /* 0x0f */
			(0xa0, 0xa0, 0xa0), /* 0x10 */
			(0x1e, 0x4a, 0x9d), /* 0x11 */
			(0x38, 0x37, 0xbc), /* 0x12 */
			(0x58, 0x28, 0xb8), /* 0x13 */
			(0x75, 0x21, 0x94), /* 0x14 */
			(0x84, 0x23, 0x5c), /* 0x15 */
			(0x82, 0x2e, 0x24), /* 0x16 */
			(0x6f, 0x3f, 0x00), /* 0x17 */
			(0x51, 0x52, 0x00), /* 0x18 */
			(0x31, 0x63, 0x00), /* 0x19 */
			(0x1a, 0x6b, 0x05), /* 0x1a */
			(0x0e, 0x69, 0x2e), /* 0x1b */
			(0x10, 0x5c, 0x68), /* 0x1c */
			(0x00, 0x00, 0x00), /* 0x1d */
			(0x00, 0x00, 0x00), /* 0x1e */
			(0x00, 0x00, 0x00), /* 0x1f */
			(0xfe, 0xff, 0xff), /* 0x20 */
			(0x69, 0x9e, 0xfc), /* 0x21 */
			(0x89, 0x87, 0xff), /* 0x22 */
			(0xae, 0x76, 0xff), /* 0x23 */
			(0xce, 0x6d, 0xf1), /* 0x24 */
			(0xe0, 0x70, 0xb2), /* 0x25 */
			(0xde, 0x7c, 0x70), /* 0x26 */
			(0xc8, 0x91, 0x3e), /* 0x27 */
			(0xa6, 0xa7, 0x25), /* 0x28 */
			(0x81, 0xba, 0x28), /* 0x29 */
			(0x63, 0xc4, 0x46), /* 0x2a */
			(0x54, 0xc1, 0x7d), /* 0x2b */
			(0x56, 0xb3, 0xc0), /* 0x2c */
			(0x3c, 0x3c, 0x3c), /* 0x2d */
			(0x00, 0x00, 0x00), /* 0x2e */
			(0x00, 0x00, 0x00), /* 0x2f */
			(0xfe, 0xff, 0xff), /* 0x30 */
			(0xbe, 0xd6, 0xfd), /* 0x31 */
			(0xcc, 0xcc, 0xff), /* 0x32 */
			(0xdd, 0xc4, 0xff), /* 0x33 */
			(0xea, 0xc0, 0xf9), /* 0x34 */
			(0xf2, 0xc1, 0xdf), /* 0x35 */
			(0xf1, 0xc7, 0xc2), /* 0x36 */
			(0xe8, 0xd0, 0xaa), /* 0x37 */
			(0xd9, 0xda, 0x9d), /* 0x38 */
			(0xc9, 0xe2, 0x9e), /* 0x39 */
			(0xbc, 0xe6, 0xae), /* 0x3a */
			(0xb4, 0xe5, 0xc7), /* 0x3b */
			(0xb5, 0xdf, 0xe4), /* 0x3c */
			(0xa9, 0xa9, 0xa9), /* 0x3d */
			(0x00, 0x00, 0x00), /* 0x3e */
			(0x00, 0x00, 0x00), /* 0x3f */
		];
		let palette_table: [u8;32] = [0;32];

        PPU {
			registers: [0;8],
			chr_rom,
			name_table: [0;2048],
			palette_table,
			ppu_status: 0
        }
    }

	// pub fn read_register(&mut self, addr: u16) -> u8 {
	// 	let result = self.registers[addr as usize];

	// 	// clear bit 7
	// 	let mut cleared_bit_7 = result;
	// 	bits::set(&mut cleared_bit_7, 7, false);
	// 	self.registers[addr as usize] = cleared_bit_7;

	// 	result
	// }

	// pub fn write_register(&mut self, addr: u16, value: u8) {
	// 	self.registers[addr as usize] = value;
	// }

	/// Returns the pattern tile at given index (0x00-0xFF) from left/right (parameter) pattern table.
	fn get_pattern_tile(&self, tile_index: u8, left_table: bool) -> &[u8] {
		if left_table {
			// Each pattern tile is 16 bytes in size. We jump by 16 bytes.
			// The tile index can be 0x0-0xFF, but the actual bytes needed are 0xFF times 16, which fits in u16.
			let i: u16 = (tile_index as u16 * 16);
			&self.chr_rom[i as usize..i as usize+16]
		} else {
			todo!();
		}
	}

	fn get_palette(&self, index: u8) {
		// Palette starts at 0x3F00 - 0x3F10 (16 bytes)
		println!("{:?}", &self.chr_rom[0x3F00..0x3F10]);
	}

}

#[cfg(test)]
mod tests {
    use crate::{cartridge::Cartridge, rom_parser::RomParser};

    use super::PPU;

	fn initialize() -> PPU {
		let path = "6502asm_programs/nestest/nestest.nes";
		let mut rom_parser = RomParser::new();
		rom_parser.parse(path);
		let cartridge: Cartridge = Cartridge::new_with_parser(rom_parser);
		let ppu = PPU::new(&cartridge);
		ppu
	}

	#[test]
	fn test_get_pattern_tile() {
		let ppu = initialize();

		let first_tile = ppu.get_pattern_tile(0, true);
		assert!(first_tile.iter().all(|&x| x == 0));

		let tile = ppu.get_pattern_tile(0xF, true);
		print_tile(tile);
		// Test the shape of the tile
		for _ in 0..16 {
			assert!(tile[3] == 0x18);
			assert!(tile[4] == 0x18);
		}
	}

	// #[test]
	// fn test_get_palette() {
	// 	let ppu = initialize();

	// 	ppu.get_palette(0);
	// }

	fn print_tile(tile: &[u8]) {
		println!("Lower 8 bytes:");
		for (i,b) in (&tile[0..8]).iter().enumerate() {
			let binary_str = format!("{:008b}", b); // pad with zeros to 8 bits
			let mut new_str = String::new();
			for bit in binary_str.chars() {
				new_str.push(bit);
				new_str.push('0');
				new_str.push(' ');
			}
			println!("Address {:#X}: \t{}\t{}", i, binary_str, new_str);
		}

		println!("Upper 8 bytes:");
		for (i,b) in (&tile[8..16]).iter().enumerate() {
			let binary_str = format!("{:008b}", b); // pad with zeros to 8 bits
			let mut new_str = String::new();
			for bit in binary_str.chars() {
				new_str.push('0');
				new_str.push(bit);
				new_str.push(' ');
			}
			println!("Address {:#X}: \t{}\t{}", i+8, binary_str, new_str);
		}

		println!("Combined and final tile:");
		for i in 0..8 {
			let lower_byte = &tile[i];
			let upper_byte = &tile[i+8];

			let binary_str_lower = format!("{:008b}", lower_byte); // pad with zeros to 8 bits
			let binary_str_upper = format!("{:008b}", upper_byte); // pad with zeros to 8 bits

			let mut new_str = String::new();
			for i in 0..8 {
				// let lower_bit = get_bit(*lower_byte, i);
				// let upper_bit = get_bit(*upper_byte, i);
				// new_str.push(char::from_digit(lower_bit as u32, 10).unwrap());
				// new_str.push(char::from_digit(upper_bit as u32, 10).unwrap());
				// new_str.push(' ');

				new_str.push(binary_str_lower.chars().nth(i).unwrap());
				new_str.push(binary_str_upper.chars().nth(i).unwrap());
				new_str.push(' ');
			}
			println!("{}", new_str);
		}
	}
}