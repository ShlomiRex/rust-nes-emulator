//#![feature(mixed_integer_ops)]  // stable since 1.67.0-nightly
mod cpu;
mod memory;
pub mod program_loader;
mod render;
mod rom_parser;
mod common;
mod ppu;
mod mapper;
mod rom;
mod cartridge;
mod mmu;

use core::panic;

use mapper::{Mapping, Mapper0};
use memory::CPUMemory;
use rom::ROM;
use cpu::cpu::CPU;
use simple_logger::SimpleLogger;
use rom_parser::RomParser;
use cartridge::Cartridge;
use mmu::MMU;

fn main() {
	SimpleLogger::new().init().unwrap();
	//render::sdl2_setup();
	
	//let path = "C:\\Users\\Shlomi\\Desktop\\Projects\\nes-test-roms\\blargg_nes_cpu_test5\\official.nes";
	//let path = "6502asm_programs/nestest.nes";
	let path = "6502asm_programs/greenscreen.nes";

	let mut rom_parser = RomParser::new();
	rom_parser.parse(path);


	let cartridge = Cartridge::new_with_parser(rom_parser);
	let mmu = MMU::new(cartridge);


	// let prg_rom = rom_parser.prg_rom;
	// let rom: ROM = ROM {
	// 	rom: prg_rom
	// };

	// let memory: CPUMemory = [0; 32768];

	// let mapper = rom_parser.header.mapper;
	// let mapper: Box<dyn Mapping> = match mapper {
	// 	0 => Box::new(Mapper0::new(memory, rom)),
	// 	_ => panic!("The emulator does not support mapper {} yet", mapper)
	// };


	let mut cpu = CPU::new(mmu);

	let mut i = 0;
	while i < 12 {
		cpu.clock_tick();
		i += 1;
	}

	// loop {
	// 	cpu.clock_tick();
	// }
}
