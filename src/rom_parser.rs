use std::fs;
use log::{debug, info};


/// Read here about iNES file format: https://www.nesdev.org/wiki/INES#iNES_file_format
#[derive(Default, Debug)]
pub struct Header {
	prg_rom_size: u8, 		// Program ROM size (in 16KB chunks, i.e., 2 means 32KB)
	chr_rom_size: u8,		// Character ROM size (in 8KN chunks)
	flags6: u8,				// Mapper, mirroring, battery, trainer

	// Flags 6 lower 4 bits ; Flags 7 upper 4 bits
	mapper: u8,

	// Flags 7
	vsunitsystem: bool,
	playchoise10: bool,
	nes2_format: bool,

	// Flags 8
	prg_ram_size: u8,

	// Flags 9
	flags9_tv_system: TVSystem,

	// Flags 10
	flags10_tv_system: TVSystem,
	prg_ram_not_present: bool,
	bus_conflicts: bool,
}

#[derive(Default, Debug)]
pub enum TVSystem {
	#[default]
	NTSC,
	PAL,
	DUAL
}

#[derive(Debug)]
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

		let flags6 = contents[6];
		let flags7 = contents[7];
		let flags8 = contents[8];
		let flags9 = contents[9];
		let flags10 = contents[10];


		// ==================== FLAGS 6 ====================
		// Mapper number (Lower 4 bits of mapper)
		let lsb_mapper = flags6 >> 4;
		
		// ==================== FLAGS 7 ====================
		// VS unitsystem
		let vsunitsystem = (flags7 & 1) == 1;

		// PlayChoice-10 (8KB of Hint Screen data stored after CHR data)
		let playchoise10 = (flags7 >> 1) & 1 == 1;

		// NES 2.0 format
		let nes2_format = (flags7 >> 2) & 0b0000_0011 == 2;
		assert_ne!(nes2_format, true, "The emulator does not support NES 2.0 format");

		// Mapper number (Upper 4 bits of mapper)
		let msb_mapper = flags7 & 0b1111_0000;

		// ==================== FLAGS 8 ====================
		// PRG RAM size
		// Size of PRG RAM in 8 KB units (Value 0 infers 8 KB for compatibility)
		let prg_ram_size = flags8;

		// ==================== FLAGS 9 ====================

		// TV system (0: NTSC; 1: PAL)
		let flags9_tv_system = if (flags9 & 1) == 1 {
			TVSystem::PAL
		} else {
			TVSystem::NTSC	
		};
		assert_eq!(flags9 >> 1, 0, "Flags 9 reserve bits are not set to zero");

		// ==================== FLAGS 10 ====================

		// TV system (0: NTSC; 2: PAL; 1/3: dual compatible)
		let flags10_tv_system = match flags10 & 0b0000_0011 {
			0 => TVSystem::NTSC,
			2 => TVSystem::PAL,
			_ => TVSystem::DUAL
		};

		// PRG RAM (0: present, 1: not present)
		let prg_ram_not_present = (flags10 >> 4) == 1;

		// Board bus conflicts (0: Board has no bus conflicts; 1: Board has bus conflicts)
		let bus_conflicts = (flags10 >> 5) == 1;


		// ==================== END ====================
		let mapper = msb_mapper | lsb_mapper;

		self.header = Header {
			prg_rom_size: 	contents[4],
			chr_rom_size: 	contents[5],
			flags6,
			mapper,
			vsunitsystem,
			playchoise10,
			nes2_format,
			prg_ram_size,
			flags9_tv_system,
			flags10_tv_system,
			prg_ram_not_present,
			bus_conflicts
		};

 		let padding_bytes = &contents[11..16];
		if padding_bytes != [0, 0, 0, 0, 0] {
			panic!("Padding bytes are not zero: {:?}", padding_bytes);
		}
		debug!("iNES header: {:#?}", self.header);
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