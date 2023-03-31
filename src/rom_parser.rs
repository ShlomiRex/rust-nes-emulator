use log::{debug, info};
use core::panic;
use std::fs;

use crate::common::{PRG_Bank, CHR_Bank};

/// Read here about iNES file format: https://www.nesdev.org/wiki/INES#iNES_file_format
#[derive(Default, Debug)]
pub struct Header {
    pub prg_rom_size: u8, // Program ROM size (in 16KB chunks, i.e., 2 means 32KB), also known as amount of banks
    pub chr_rom_size: u8, // Character ROM size (in 8KN chunks), also known as amount of banks
    pub mapper: u8,

    // Flags 6
    pub mirroring: MirrorType,
    pub battery_prg_ram: bool,
    pub trainer: bool,
    ignore_mirroring_control: bool,

    // Flags 7
    vs_unit_system: bool,
    play_choise_10: bool,
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
    DUAL,
}

#[derive(Default, Debug, Clone)]
pub enum MirrorType {
    #[default]
    HORIZONTAL,
    VERTICAL,
}

#[derive(Debug)]
pub struct RomParser {
    pub header: Header,
    pub prg_rom: Vec<PRG_Bank>,
    pub chr_rom: Vec<CHR_Bank>,
}

impl RomParser {
    pub fn new() -> Self {
        RomParser {
            header: Header::default(),
            prg_rom: vec![],
            chr_rom: vec![],
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
        // Mirroring: 	0: horizontal (vertical arrangement) (CIRAM A10 = PPU A11)
        // 				1: vertical (horizontal arrangement) (CIRAM A10 = PPU A10)
        let mirroring = if (flags6 & 1) == 1 {
			MirrorType::VERTICAL
		} else {
			MirrorType::HORIZONTAL
		};

		// 1: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
		let battery_prg_ram = (flags6 >> 1) & 1 == 1;

        // 1: 512-byte trainer at $7000-$71FF (stored before PRG data)
        let trainer = (flags6 >> 2) & 1 == 1;

        // 1: Ignore mirroring control or above mirroring bit; instead provide four-screen VRAM
        let ignore_mirroring_control = (flags6 >> 3) & 1 == 1;

        // Mapper number (Lower 4 bits of mapper)
        let lsb_mapper = flags6 >> 4;

        // ==================== FLAGS 7 ====================
        // VS unitsystem
        let vs_unit_system = (flags7 & 1) == 1;

        // PlayChoice-10 (8KB of Hint Screen data stored after CHR data)
        let play_choise_10 = (flags7 >> 1) & 1 == 1;

        // NES 2.0 format
        let nes2_format = (flags7 >> 2) & 0b0000_0011 == 2;
        assert_ne!(
            nes2_format, true,
            "The emulator does not support NES 2.0 format"
        );

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
            _ => TVSystem::DUAL,
        };

        // PRG RAM (0: present, 1: not present)
        let prg_ram_not_present = (flags10 >> 4) == 1;

        // Board bus conflicts (0: Board has no bus conflicts; 1: Board has bus conflicts)
        let bus_conflicts = (flags10 >> 5) == 1;

        // ==================== END ====================
        let mapper = msb_mapper | lsb_mapper;

		if mapper != 0 {
			panic!("The emulator only supports mapper 0. ROM mapper is {}", mapper);
		}

        self.header = Header {
            prg_rom_size: contents[4],
            chr_rom_size: contents[5],
            mapper,
            mirroring,
            battery_prg_ram,
            trainer,
            ignore_mirroring_control,
            vs_unit_system,
            play_choise_10,
            nes2_format,
            prg_ram_size,
            flags9_tv_system,
            flags10_tv_system,
            prg_ram_not_present,
            bus_conflicts,
        };

        let padding_bytes = &contents[11..16];
        if padding_bytes != [0, 0, 0, 0, 0] {
            panic!("Padding bytes are not zero: {:?}", padding_bytes);
        }
        debug!("iNES header: {:#?}", self.header);
    }

    fn parse_prg_rom(&mut self, contents: &Vec<u8>) {
        let prg_rom_size_bytes: usize = 1024 * 16 * self.header.prg_rom_size as usize;

		// The entire PRG memory in one vector
        let prg_rom = &contents[16..16 + prg_rom_size_bytes];

        debug!("PRG ROM size: {}KB", prg_rom.len()/1024);
        //assert_eq!(prg_rom.len(), 1024 * 32, "The emulator, currently, supports PRG ROM of size 32KB.");
        debug!("First 16 bytes of PRG ROM: {:X?}", &prg_rom[0..16]);
        debug!(
            "Last 16 bytes of PRG ROM: {:X?}",
            &prg_rom[prg_rom.len() - 16..]
        ); // This contains interrupt vectors

		// Split the PRG memory into banks
		self.prg_rom = Vec::with_capacity(self.header.prg_rom_size as usize);
		for chunk in prg_rom.chunks_exact(16 * 1024) {
			self.prg_rom.push(chunk.to_vec().try_into().unwrap());
		}
    }

    fn parse_chr_rom(&mut self, contents: &Vec<u8>) {
        let mut chr_rom_bytes = 1024 * 8 * self.header.chr_rom_size as usize;
        debug!("CHR ROM size: {}KB", chr_rom_bytes/1024);
        if chr_rom_bytes == 0 {
            //TODO: What to do here? It's empty bytes.
            // "If Y=0, you prepare an empty 8192 bytes of memory, and allow writing into CHR."
            // Quote from my question on reddit: https://www.reddit.com/r/EmuDev/comments/yvaz54/comment/iwdq3s9/?utm_source=share&utm_medium=web2x&context=3
            info!("CHR ROM size is 0, preparing empty 8192 bytes of memory");
            chr_rom_bytes = 1024 * 8;
        }
        let prg_rom_size_bytes: usize = 1024 * 16 * self.header.prg_rom_size as usize;
        let chr_rom = &contents[16 + prg_rom_size_bytes..]; // Get the rest of the bytes. The size of the entire file must be exact match to expected size.
        assert_eq!(chr_rom.len(), chr_rom_bytes);

		// Split the CHR memory into banks
		self.chr_rom = Vec::with_capacity(self.header.chr_rom_size as usize);
		for chunk in chr_rom.chunks_exact(8 * 1024) {
			self.chr_rom.push(chunk.to_vec().try_into().unwrap());
		}
    }
}
