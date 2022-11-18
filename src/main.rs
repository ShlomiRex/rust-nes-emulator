//#![feature(mixed_integer_ops)]  // stable since 1.67.0-nightly
mod cpu;
mod memory;
pub mod program_loader;
mod render;
mod rom_parser;

use log::info;
use memory::{ROM, Memory, MemoryBus};
use cpu::cpu::CPU;
use render::sdl2_setup;
use simple_logger::SimpleLogger;
use rom_parser::RomParser;

fn greenscreen() {
	let path = "6502asm_programs/greenscreen.nes";
	let mut rom_parser = RomParser::new();
	rom_parser.parse(path);
	let prg_rom = rom_parser.prg_rom;
	
	let rom: ROM = ROM {
		rom: prg_rom
	};

	let memory: Memory = [0; 32768];
	let memory_bus = MemoryBus::new(memory, rom);
	let mut cpu = CPU::new(memory_bus);

	sdl2_setup();

	for i in 0..20 {
		info!("Assembly line: {}", i);
		cpu.clock_tick();
	}
}

fn main() {
	SimpleLogger::new().init().unwrap();

	greenscreen();
	
	let path = "C:\\Users\\Shlomi\\Desktop\\Projects\\nes-test-roms\\blargg_nes_cpu_test5\\cpu.nes";
	let mut rom_parser = RomParser::new();
	rom_parser.parse(path);
	let prg_rom = rom_parser.prg_rom;
	
	let rom: ROM = ROM {
		rom: prg_rom
	};

	let memory: Memory = [0; 32768];
	let memory_bus = MemoryBus::new(memory, rom);
	let mut cpu = CPU::new(memory_bus);

	loop {
		cpu.clock_tick();
	}
}
