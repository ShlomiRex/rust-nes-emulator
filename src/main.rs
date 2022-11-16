//#![feature(mixed_integer_ops)]  // stable since 1.67.0-nightly
mod cpu;
mod memory;
pub mod program_loader;
mod render;
mod rom_parser;

use log::info;
use memory::{ROM, Memory, MemoryBus};
use cpu::cpu::CPU;
use simple_logger::SimpleLogger;
use rom_parser::RomParser;

fn main() {
	SimpleLogger::new().init().unwrap();

	//let path = "6502asm_programs/nestest.nes";
	//let path = "6502asm_programs/minimal.nes";
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

	let mut i = 1;
	loop {
		info!("Assembly line: {}", i);
		cpu.clock_tick();
		i += 1;
	};
}
