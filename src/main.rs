//#![feature(mixed_integer_ops)]  // stable since 1.67.0-nightly

use log::info;
use simple_logger::SimpleLogger;

mod cpu;
mod bus;
mod memory;
mod program_loader;

use bus::Bus;
use memory::ROM;
use cpu::cpu::CPU;
use program_loader::*;

fn main() {
	SimpleLogger::new().init().unwrap();

	let mut rom_memory: [u8; 65_536] = [0;65_536];

	let assembly_lines_amount = load_program_reset_sp(&mut rom_memory);
	
	let rom: ROM = ROM {
		rom: Box::new(rom_memory)
	};
	
	let bus = Box::new(Bus::new(rom));
	let mut cpu = CPU::new(bus);

	for _ in 0..assembly_lines_amount {
		cpu.clock_tick();
	}

	info!("Finished running NES");
}
