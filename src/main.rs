use simple_logger::SimpleLogger;
use log::{error, warn, debug, info};

mod cpu;
mod bus;
mod memory;

use crate::cpu::CPU;
use bus::Bus;
use memory::{ROM, RAM};


// https://web.archive.org/web/20210803073202/http://www.obelisk.me.uk/6502/architecture.html
// Zero page: 0x0000 - 0x00FF : is the focus of a number of special addressing modes that result in shorter (and quicker) instructions or allow indirect access to the memory (256 bytes of memory)
// Stack: 	  0x0100 - 0x01FF : is reserved for the system stack and which cannot be relocated. (256 bytes of stack!)
// Reserved memory: 0xFFFA - 0xFFFF (last 6 bytes) : must be programmed with the addresses of the non-maskable interrupt handler ($FFFA/B), the power on reset location ($FFFC/D) and the BRK/interrupt request handler ($FFFE/F) respectively.

fn main() {
	SimpleLogger::new().init().unwrap();

	// Here we can write to ROM memory and modify the instructions to execute.
	// The example here is from wiki page: https://en.wikipedia.org/wiki/MOS_Technology_6502
	let mut rom_memory: [u8; 65_536] = [0;65_536];
	rom_memory[0] = 0xA0;
	rom_memory[1] = 0x00;
	rom_memory[2] = 0xB1;
	rom_memory[3] = 0x80;
	rom_memory[4] = 0xF0;
	rom_memory[5] = 0x11;
	

	let a = Box::new(rom_memory);
	let rom: ROM = ROM {
		rom: a
	};
	
	let bus = Box::new(Bus::new(rom));
	let mut cpu = CPU::new(bus);

	cpu.clock_tick();
	cpu.clock_tick();
	cpu.clock_tick();
	cpu.clock_tick();
	cpu.clock_tick();
}
