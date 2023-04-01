//#![feature(mixed_integer_ops)]  // stable since 1.67.0-nightly
mod cpu;
pub mod program_loader;
mod render;
mod rom_parser;
mod common;
mod ppu;
mod cartridge;
mod nes;

use std::io;

use nes::NES;
use simple_logger::SimpleLogger;

fn main() {
	SimpleLogger::new().init().unwrap();
	//render::sdl2_setup();
	
	//let path = "C:\\Users\\Shlomi\\Desktop\\Projects\\nes-test-roms\\blargg_nes_cpu_test5\\official.nes";
	let path = "6502asm_programs/nestest/nestest.nes";
	//let path = "6502asm_programs/greenscreen.nes";
	//let path = "6502asm_programs/background/background.nes";

	let mut nes = NES::new_open_rom_file(path);

	// let mut i = 0;
	// while i < 12 {
	// 	cpu.clock_tick();
	// 	i += 1;
	// }

	let allow_stepping = true;
	let stdin = io::stdin();
	
	loop {
		if allow_stepping {
			// Read and discard
			let mut buf: String = String::new();
			let _ = stdin.read_line(&mut buf).unwrap();
		}
		nes.cpu.clock_tick();
		//std::thread::sleep(std::time::Duration::from_millis(200));
	}
}
