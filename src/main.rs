//#![feature(mixed_integer_ops)]  // stable since 1.67.0-nightly
mod cpu;
mod bus;
mod memory;
mod program_loader;
mod ppu;
mod render;

use log::info;
use simple_logger::SimpleLogger;
use bus::Bus;
use memory::ROM;
use cpu::cpu::CPU;
use program_loader::*;
use render::sdl2_setup;

fn main() {
	SimpleLogger::new().init().unwrap();

	{
		// Create ROM and load it with simple program.
		let mut rom_memory: [u8; 65_536] = [0;65_536];
		let assembly_lines_amount = load_program_zeropage_x(&mut rom_memory);
		let rom: ROM = ROM {
			rom: Box::new(rom_memory)
		};
		
		// Create CPU.
		let bus = Box::new(Bus::new(rom));
		let mut cpu = CPU::new(bus);

		// Execute clocks.
		for _ in 0..assembly_lines_amount {
			cpu.clock_tick();
		}

		info!("Finished running NES");
	}

	sdl2_setup();
}
