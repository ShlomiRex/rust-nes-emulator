mod cpu;
mod bus;

use cpu::{Registers, CPU};
use bus::Bus;
use bus::Address;



fn initialize() {
	let ram: Box<[u8; 65536]> = Box::new([0; 65536]);
	let mut bus = Bus {
		RAM: ram
	};

}

fn main() {
	initialize();
}

#[cfg(test)]
mod tests {
	use super::*;

    #[test]
    fn ram_test() {
		let ram: Box<[u8; 65536]> = Box::new([0; 65536]);
		let mut bus = Bus {
			RAM: ram
		};

		let addr: Address = 0xFFFF;
		assert_eq!(bus.read(addr), 0x00);
		bus.write(addr, 0xFF);
		assert_eq!(bus.read(addr), 0xFF);
    }

	#[test]
	fn cpu_test() {
		let cpu = Registers { ..Default::default() };
		println!("{} {} {:?}", cpu.A, cpu.PC, cpu.P);
	}
}