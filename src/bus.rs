use log::debug;

use crate::memory::{MemoryBus, ROM};

/// Bus is like a container that glue every component together, like on the motherboard.
pub struct Bus {
	pub memory: MemoryBus,
	rom: ROM
}

impl Bus {
	pub fn new(rom: ROM) -> Self {
		Bus { 
			memory: MemoryBus::new(), 
			rom: rom
		}
	}

	/// Maps PRG ROM onto memory (for now its the last 32kb)
	pub fn map_prg_rom(&mut self) {
		for i in 0x8000..0xFFFF + 1 {
			self.memory.memory[i] = self.rom.rom[i - 0x8000];
		}
	}
}