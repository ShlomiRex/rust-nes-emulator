use crate::memory::{RAM, ROM};

/// Bus is like a container that glue every component together, like on the motherboard.
pub struct Bus {
	pub ram: RAM,
	pub rom: ROM		// NOTE: The ROM can be as large as 8MB. For now, its 64kb just so I have MVP (minimal viable product).
}

impl Bus {
	pub fn new(rom: ROM) -> Self {
		Bus { 
			ram: RAM::new(), 
			rom: rom
		}
	}
}