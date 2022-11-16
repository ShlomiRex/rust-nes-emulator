//#![feature(mixed_integer_ops)]  // stable since 1.67.0-nightly
mod cpu;
mod bus;
mod memory;
pub mod program_loader;
//mod ppu;
mod render;
mod rom_parser;

use log::debug;
use bus::Bus;
use memory::ROM;
use cpu::cpu::CPU;
// use program_loader::*;
use simple_logger::SimpleLogger;
use rom_parser::RomParser;

// fn basic_run() {
// 	// Create ROM and load it with simple program.
// 	let mut rom_memory: [u8; 65_536] = [0;65_536];
// 	let assembly_lines_amount = load_program_zeropage_x(&mut rom_memory);
// 	let rom: ROM = ROM {
// 		rom: Box::new(rom_memory)
// 	};
	
// 	// Create CPU.
// 	let bus = Box::new(Bus::new(rom));
// 	let mut cpu = CPU::new(bus);

// 	// Execute clocks.
// 	for _ in 0..assembly_lines_amount {
// 		cpu.clock_tick();
// 	}

// 	info!("Finished running NES");
// }

fn main() {
	SimpleLogger::new().init().unwrap();

	let path = "6502asm_programs/minimal.nes";
	let mut rom_parser = RomParser::new();
	rom_parser.parse(path);
	let prg_rom = rom_parser.prg_rom;
	
	let rom: ROM = ROM {
		rom: prg_rom
	};
	let mut bus = Box::new(Bus::new(rom));
	bus.map_prg_rom();

	let mut cpu = CPU::new(bus.memory);
	cpu.clock_tick();
	cpu.clock_tick();
	cpu.clock_tick();
	cpu.clock_tick();
	cpu.clock_tick();
	cpu.clock_tick();
	

}
