use std::fs;
use log::{debug, info};

#[derive(Default)]
struct Header {
	prg_rom_size: u8,
	chr_rom_size: u8,
	flags6: u8,
	flags7: u8,
	flags8: u8,
	flags9: u8,
	flags10: u8
}

pub struct RomParser {
	header: Header
}

impl RomParser {
	pub fn new() -> Self {
		RomParser {
			header: Header::default()
		}
	}

	pub fn parse(&mut self, path: &str) {
		debug!("Parsing ROM: {}", path);
		let contents = fs::read(path).expect("Could not read ROM");

		self.parse_header(&contents[0..16].try_into().unwrap());
	}

	fn parse_header(&mut self, header_bytes: &[u8; 16]) {
		assert_eq!(&header_bytes[0..4], b"NES\x1A", "Incorrect magic bytes");

		self.header = Header {
			prg_rom_size: header_bytes[4],
			chr_rom_size: header_bytes[5],
			flags6: header_bytes[6],
			flags7: header_bytes[7],
			flags8: header_bytes[8],
			flags9: header_bytes[9],
			flags10: header_bytes[10]
		};

		let padding_bytes = &header_bytes[11..];
		if padding_bytes != [0, 0, 0, 0, 0] {
			debug!("Padding bytes are not zero: {:?}", padding_bytes);
		}
	}
}