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

use mapper::Mapper0;
use memory::Memory;
use rom::ROM;
use cpu::cpu::CPU;
use simple_logger::SimpleLogger;
use rom_parser::RomParser;


fn main() {
	SimpleLogger::new().init().unwrap();

	//greenscreen();
	
	let path = "C:\\Users\\Shlomi\\Desktop\\Projects\\nes-test-roms\\blargg_nes_cpu_test5\\official.nes";
	//let path = "6502asm_programs/nestest.nes";
	//let path = "6502asm_programs/greenscreen.nes";

	let mut rom_parser = RomParser::new();
	rom_parser.parse(path);

	// TODO: Check mapper support, currently i'm working on mapper 1
	let mapper = rom_parser.header.mapper;
	if mapper != 0 && mapper != 1 {
		panic!("The emulator does not support mapper {} yet", mapper);
	}


	let prg_rom = rom_parser.prg_rom;
	
	let rom: ROM = ROM {
		rom: prg_rom
	};

	let memory: Memory = [0; 32768];
	let mapper0 = Mapper0::new(memory, rom);
	let mut cpu = CPU::new(mapper0);

	let mut i = 0;
	while i < 12 {
		cpu.clock_tick();
		i += 1;
	}
	// loop {
	// 	cpu.clock_tick();
	// }
}
