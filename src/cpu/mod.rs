mod registers;
mod decoder;

use log::{debug};

use registers::Registers;
use crate::bus::Bus;
use decoder::CycleWare;

/// # 8-bit databus
/// Not to be confused with 'the bus', the data bus is 8 bits (8 input signals).
struct DataBus;
/// This signal allows to halt or single cycle the microprocessor on all cycles except write cycles.
struct ReadyInputSignal;
/// Request that an interrupt sequence begin within the microprocessor.
struct InterruptRequestSignal;
/// Request that a non-maskable interrupt sequence be generated within the microprocessor.
struct NonMaskableInterruptSignal;
/// Sets the overflow bit in the Processor Status Register.
struct SetOverflowSignal;

pub struct CPU {
	registers: Registers,
	bus: Box<Bus>
}

impl CPU {
	pub fn new(bus: Box<Bus>) -> Self {
		let registers: Registers = Registers::default();
		CPU {
			registers,
			bus
		}
	}

	/// A single clock cycle is executed here.
	pub fn clock_tick(&mut self) {
		//TODO: Complete

		// Read next instruction.
		let opcode = self.bus.rom.read(self.registers.PC); // Read at address of Program Counter (duh!)
		let instruction = decoder::decode_opcode(opcode);

		let instr = instruction.0;
		let addrmode = instruction.1;
		let bytes = instruction.2;
		let cycles = instruction.3;
		let cycle_ware = instruction.4;

		debug!("{:?} {:?} Bytes: {}, Cycles: {}, Cycles mut: {:?}", instr, addrmode, bytes, cycles, cycle_ware);

		// Continue the program counter
		match cycle_ware {
			CycleWare::NONE => { 
				self.registers.PC += bytes as u16; 
			},
			CycleWare::PageBoundryCrossed => { 
				//TODO: Impliment. For now, I don't change amount of cycles.
				//add 1 to cycles if page boundary is crossed
				self.registers.PC += bytes as u16; 
			},
			CycleWare::BranchOccursOn => {
				//TODO: Impliment. For now, I don't change amount of cycles.
				//add 1 to cycles if branch occurs on same page
				//add 2 to cycles if branch occurs to different page
				self.registers.PC += bytes as u16;
			}
		}
				
	}

	fn nmi_interrupt(&self) {
		//TODO: Complete
	}

	fn irq_interrupt(&self) {
		//TODO: Complete
	}
}

#[cfg(test)]
mod tests {
    use super::*;
	use registers::ProcessorStatusRegisterBits::*;
	use crate::bus::Bus;
	use crate::memory::ROM;

    #[test]
    fn processor_status_register_test() {
		//let rom = ROM::new();
		let rom = ROM::new(Box::new([0;65_536]));
		let bus: Bus = Bus::new(rom);
		let mut cpu = CPU::new(Box::new(bus));

		assert!(cpu.registers.P.get(CARRY) == false);
		cpu.registers.P.set(CARRY, true);
		assert!(cpu.registers.P.get(CARRY) == true);

		assert!(cpu.registers.P.get(NEGATIVE) == false);
		cpu.registers.P.set(NEGATIVE, true);
		assert!(cpu.registers.P.get(NEGATIVE) == true);
		cpu.registers.P.set(NEGATIVE, false);
		assert!(cpu.registers.P.get(NEGATIVE) == false);
		cpu.registers.P.set(NEGATIVE, false);
		assert!(cpu.registers.P.get(NEGATIVE) == false);
    }
}