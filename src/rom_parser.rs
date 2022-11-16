use std::fs;
use log::{debug, info};


/// Read here about iNES file format: https://www.nesdev.org/wiki/INES#iNES_file_format
#[derive(Default, Debug)]
pub struct Header {
	prg_rom_size: u8, 		// Program ROM size (in 16KB chunks, i.e., 2 means 32KB)
	chr_rom_size: u8,		// Character ROM size (in 8KN chunks)
	flags6: u8,				// Mapper, mirroring, battery, trainer
	flags7: u8,				// Mapper, VS/Playchoice, NES 2.0
	flags8: u8,				// PRG-RAM size (rarely used extension)
	flags9: u8,				// TV system (rarely used extension)
	flags10: u8				// TV system, PRG-RAM presence (unofficial, rarely used extension)
}

pub struct RomParser {
	pub header: Header,
	pub prg_rom: Vec<u8>,
	pub chr_rom: Vec<u8>
}

impl RomParser {
	pub fn new() -> Self {
		RomParser {
			header: Header::default(),
			prg_rom: vec![],
			chr_rom: vec![]
		}
	}

	pub fn parse(&mut self, path: &str) {
		info!("Parsing ROM: {}", path);
		let contents = fs::read(path).expect("Could not read NES ROM");

		self.parse_header(&contents);
		self.parse_prg_rom(&contents);
		self.parse_chr_rom(&contents);
	}

	fn parse_header(&mut self, contents: &Vec<u8>) {
		assert_eq!(&contents[0..4], b"NES\x1A", "Incorrect magic bytes");

		self.header = Header {
			prg_rom_size: 	contents[4],
			chr_rom_size: 	contents[5],
			flags6: 		contents[6],
			flags7: 		contents[7],
			flags8: 		contents[8],
			flags9: 		contents[9],
			flags10: 		contents[10]
		};

 		let padding_bytes = &contents[11..16];
		if padding_bytes != [0, 0, 0, 0, 0] {
			debug!("Padding bytes are not zero: {:?}", padding_bytes);
		}
		debug!("{:#02X?}", self.header);
	}

	fn parse_prg_rom(&mut self, contents: &Vec<u8>) {
		let prg_rom_size_bytes: usize = 1024 * 16 * self.header.prg_rom_size as usize;
		let prg_rom = &contents[16..16 + prg_rom_size_bytes];
		debug!("PRG ROM bytes: {}", prg_rom.len());
		//assert_eq!(prg_rom.len(), 1024 * 32, "The emulator, currently, supports PRG ROM of size 32KB.");
		debug!("First 16 bytes of PRG ROM: {:X?}", &prg_rom[0..16]);
		debug!("Last 16 bytes of PRG ROM: {:X?}", &prg_rom[prg_rom.len()-16..]);  // This contains interrupt vectors
		self.prg_rom = prg_rom.to_vec();
	}

	fn parse_chr_rom(&mut self, contents: &Vec<u8>) {
		let mut chr_rom_bytes = 1024 * 8 * self.header.chr_rom_size as usize;
		debug!("CHR ROM bytes: {}", chr_rom_bytes);
		if chr_rom_bytes == 0 {
			//TODO: What to do here? It's empty bytes.
			// "If Y=0, you prepare an empty 8192 bytes of memory, and allow writing into CHR."
			// Quote from my question on reddit: https://www.reddit.com/r/EmuDev/comments/yvaz54/comment/iwdq3s9/?utm_source=share&utm_medium=web2x&context=3
			info!("CHR ROM size is 0, preparing empty 8192 bytes of memory");
			chr_rom_bytes = 1024 * 8;
		}
		let prg_rom_size_bytes: usize = 1024 * 16 * self.header.prg_rom_size as usize;
		let chr_rom = &contents[16 + prg_rom_size_bytes..]; 		// Get the rest of the bytes. The size of the entire file must be exact match to expected size.
		assert_eq!(chr_rom.len(), chr_rom_bytes);
		self.chr_rom = chr_rom.to_vec();
	}

}