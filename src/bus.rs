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
}