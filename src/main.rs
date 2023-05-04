//#![feature(mixed_integer_ops)]  // stable since 1.67.0-nightly
mod cartridge;
mod common;
mod cpu;
mod nes;
mod ppu;
pub mod program_loader;
mod render;
mod rom_parser;

use std::io;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Mutex, Arc};

use nes::NES;
use simple_logger::SimpleLogger;
use log::{debug, info};

fn main() {
    SimpleLogger::new().init().unwrap();

	let closed_window_mutex = Arc::new(Mutex::new(false));
	let closed_window_mutex_clone = Arc::clone(&closed_window_mutex);
	// Create thread for handling drawing/graphics, the NES is executed on main thread
    let handle = thread::spawn(move || {
        render::sdl2_setup();

		// Set flag that the SDL window finished
		let mut value = closed_window_mutex_clone.lock().unwrap();
        *value = true;
    });

    //let path = "C:\\Users\\Shlomi\\Desktop\\Projects\\nes-test-roms\\blargg_nes_cpu_test5\\official.nes";
    let path = "6502asm_programs/nestest/nestest.nes";
    //let path = "6502asm_programs/greenscreen.nes";
    //let path = "6502asm_programs/background/background.nes";

    let mut nes = NES::new_open_rom_file(path);

    let allow_stepping = false;
    let stdin = io::stdin();


    loop {
		let value = closed_window_mutex.lock().unwrap();
        if *value {
            break;
        }
		drop(value);

        if allow_stepping {
            // Read and discard
            let mut buf: String = String::new();
            let _ = stdin.read_line(&mut buf).unwrap();
        }
        nes.cpu.clock_tick();
        //std::thread::sleep(std::time::Duration::from_millis(200));
    }

	// Wait for the thread to finish executing
	handle.join().expect("Failed to join the thread.");

}
