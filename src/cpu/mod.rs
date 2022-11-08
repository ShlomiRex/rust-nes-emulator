mod registers;
mod decoder;

use core::panic;

use log::{debug, error};

use registers::Registers;
use registers::ProcessorStatusRegisterBits;
use crate::bus::Bus;
use crate::cpu::decoder::AddressingMode;
use decoder::{CyclesMutations, Instructions};

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
	bus: Box<Bus>,
	cycles: u64
}


impl CPU {
	pub fn new(bus: Box<Bus>) -> Self {
		let registers: Registers = Registers::default();
		CPU {
			registers,
			bus,
			cycles: 0
		}
	}

	/// A single clock cycle is executed here.
	pub fn clock_tick(&mut self) {
		debug!("Tick, cycle: {}", self.cycles);
		debug!("{}", self.registers);

		// Read next instruction.
		let opcode = self.bus.rom.read(self.registers.PC); // Read at address of Program Counter (duh!)
		let instruction = decoder::decode_opcode(opcode);

		let instr = instruction.0;
		let addrmode = instruction.1;
		let bytes = instruction.2;
		let cycles = instruction.3;
		let cycles_mut = instruction.4;

		debug!("{:#X}: {:?}\t{:?}\tBytes: {}, Cycles: {}, Cycles mut: {}", opcode, instr, addrmode, bytes, cycles, cycles_mut);

		match cycles_mut {
			CyclesMutations::NONE => { 
				// don't change amount of cycles.
			},
			CyclesMutations::PageBoundryCrossed => { 
				//TODO: Impliment. For now, I don't change amount of cycles.

				//add 1 to cycles if page boundary is crossed
			},
			CyclesMutations::BranchOccursOn => {
				//TODO: Impliment. For now, I don't change amount of cycles.

				//add 1 to cycles if branch occurs on same page
				//add 2 to cycles if branch occurs to different page
			}
		}

		//Fetch the needed memory for the instruction
		let fetched_memory = match addrmode {
			AddressingMode::IMMEDIATE => {
				self.fetch_immediate()
			}
			// AddressingMode::INDIRECTY => {
			// 	self.fetch_indirect_y()
			// }
			_ => {
				error!("Address mode: {:?} is currently not implemented", addrmode);
				panic!();
			}
		};

		//The main brains of the CPU. Execute instruction.
		//TODO: optimize order of matching
		//By ordering the most used instructions first, I can optimize this code. But I'm only starting so its not relevant at all.
		match instr {
			Instructions::LDY => {
				// Load Index Y with Memory
				self.registers.Y = fetched_memory;
			}
			Instructions::LDA => {
				// Load Accumulator with Memory
				self.registers.A = fetched_memory;
			}
			_ => {
				error!("Could not execute instruction: {:?}, not implimented, yet", instr);
				panic!();
			}
		}

		// Increment PC by amount of bytes needed for the instruction, other than opcode (which is 1 byte).
		// We do this at the end of the execution, because we need to access the PC (for the current instruction) before we increment it.
		self.registers.PC += bytes as u16;

		self.cycles += 1;
	}

	//TODO: Optimize all fetch functions as inline?


	/// Fetch a single byte of immediate memory.
	fn fetch_immediate(&self) -> u8 {
		let res = self.bus.rom.read(self.registers.PC + 1);
		debug!("Fetched immediate: {}", res);
		res
	}

	// fn fetch_indirect_y(&self) -> u8 {
	// 	//TODO: Impliment
	// }

	fn nmi_interrupt(&self) {
		//TODO: Complete
	}

	fn irq_interrupt(&self) {
		//TODO: Complete
	}

	fn reset_interrupt(&self) {
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