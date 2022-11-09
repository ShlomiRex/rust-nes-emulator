mod registers;
mod decoder;

use core::panic;

use log::{debug, error};

use registers::Registers;
use crate::bus::Bus;
use crate::cpu::decoder::AddressingMode;
use decoder::{CycleOops, Instructions};
use registers::ProcessorStatusRegisterBits;

// /// # 8-bit databus
// /// Not to be confused with 'the bus', the data bus is 8 bits (8 input signals).
// struct DataBus;
// /// This signal allows to halt or single cycle the microprocessor on all cycles except write cycles.
// struct ReadyInputSignal;
// /// Request that an interrupt sequence begin within the microprocessor.
// struct InterruptRequestSignal;
// /// Request that a non-maskable interrupt sequence be generated within the microprocessor.
// struct NonMaskableInterruptSignal;
// /// Sets the overflow bit in the Processor Status Register.
// struct SetOverflowSignal;

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
		let cycle_oops = instruction.4;
		let p_bits_change = instruction.5;

		debug!("{:#X}: {:?}\t{:?}\tBytes: {}, Cycles: {}, Oops cycle: {}, P modify: {}", opcode, instr, addrmode, bytes, cycles, cycle_oops, p_bits_change);

		match cycle_oops {
			CycleOops::NONE => { 
				// don't change amount of cycles.
			},
			CycleOops::PageBoundryCrossed => { 
				//TODO: Impliment. For now, I don't change amount of cycles.

				//add 1 to cycles if page boundary is crossed
			},
			CycleOops::BranchOccursOn => {
				//TODO: Impliment. For now, I don't change amount of cycles.

				//add 1 to cycles if branch occurs on same page
				//add 2 to cycles if branch occurs to different page
			}
		}

		fn panic_addressing_mode_unsupported(addrmode: AddressingMode) {
			error!("The instruction doesn't support addressing mode: {:?}, panic", addrmode);
			panic!();
		}

		// Here we allowed to cast u8 to u16 because its the Address bus. The CPU only supports address bus of 16 signals (bits).
		let fetched_memory: u16 = match addrmode {
			AddressingMode::IMPLIED => 0, // Implied means this instruction doesn't fetch any memory. For now its zero. It won't be used.
			AddressingMode::ABSOLUTE => self.fetch_absolute(),
			AddressingMode::RELATIVE => self.fetch_relative(),
			AddressingMode::IMMEDIATE => self.fetch_immediate() as u16,
			AddressingMode::ACCUMULATOR => self.fetch_accumulator() as u16,
			AddressingMode::INDIRECTY => self.fetch_indirect_y(),
			
			_ => {
				panic_addressing_mode_unsupported(addrmode);
				panic!();
			}
		};

		// After each instruction, the P register may change, depending on the instruction.
		// The instruction sets the desired bit flag to change, and after execution, the CPU checks and modifies the P register.
		// Each vector contains 0 or more bits to modify/set/clear and so on.
		// TODO: Optimize by moving vectors to struct, and not initialize them each clock tick. Also, clear them after cycle.
		let mut p_modify: Vec<ProcessorStatusRegisterBits> = Vec::new();
		let mut p_clear: Vec<ProcessorStatusRegisterBits> = Vec::new();
		let mut p_set: Vec<ProcessorStatusRegisterBits> = Vec::new();

		//The main brains of the CPU. Execute instruction.
		//TODO: optimize order of matching
		//By ordering the most used instructions first, I can optimize this code. But I'm only starting so its not relevant at all.
		match instr {
			Instructions::LDY => {
				// Load Index Y with Memory
				self.registers.Y = fetched_memory as u8;

				p_modify.push(ProcessorStatusRegisterBits::ZERO);
				p_modify.push(ProcessorStatusRegisterBits::NEGATIVE);
			}
			Instructions::LDA => {
				// Load Accumulator with Memory
				self.registers.A = fetched_memory as u8;

				p_modify.push(ProcessorStatusRegisterBits::ZERO);
				p_modify.push(ProcessorStatusRegisterBits::NEGATIVE);
			}
			_ => {
				error!("Could not execute instruction: {:?}, not implimented, yet", instr);
				panic!();
			}
		}

		// Modify P register.

		for p_bit in p_clear {
			self.registers.P.set(p_bit, false);
		}
		for p_bit in p_set {
			self.registers.P.set(p_bit, true);
		}
		for p_bit in p_modify {
			match p_bit {
				ProcessorStatusRegisterBits::ZERO => { 
					// If memory is 0, zero flag is 1
					self.registers.P.set(p_bit, fetched_memory == 0); 
				}
				ProcessorStatusRegisterBits::NEGATIVE => { 
					// If last bit (7) is 1, its negative
					self.registers.P.set(ProcessorStatusRegisterBits::NEGATIVE, (fetched_memory >> 7) == 1);
				}
				_ => {
					panic!("The P bit to modify is unsupported: {:?}", p_bit);
				}
			}
		}

		// Increment PC by amount of bytes needed for the instruction, other than opcode (which is 1 byte).
		// We do this at the end of the execution, because we need to access the PC (for the current instruction) before we increment it.
		// For example, when we have LDA, we load A with immediate memory at the next byte of PC. So we access PC + 1.
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

	/// Fetch memory from accumulator.
	fn fetch_accumulator(&self) -> u8 {
		let res = self.registers.A;
		debug!("Fetched accumulator: {}", res);
		res
	}

	fn processor_status_flag_modified(&self, bit: ProcessorStatusRegisterBits) {

	}

	/// Fetch memory from the next 2 bytes of the instruction. (after opcode)
	/// The memory is 2 bytes because this is the address size in 6502.
	fn fetch_absolute(&self) -> u16 {
		let res = self.bus.ram.read_address(self.registers.PC);
		debug!("Fetched absolute: {:X}", res);
		res
	}

	/// Relative addressing is PC + offset.
	/// The offset is the next byte after opcode.
	/// So we fetch the next byte (after opcode) and add it with PC.
	/// IMPORTANT: The offset is SIGNED. Which means, the offset can be -128 to 127.
	fn fetch_relative(&self) -> u16 {
		let pc = self.registers.PC;
		let offset = self.bus.rom.read(self.registers.PC + 1) as i8;
		// Here we need a way to add 'u16' type with 'i8' type.
		// IMPORTANT NOTE: We need the "mixed_integer_ops" feature, which is in nightly rust.
		// Its very complex to do this manually, without this feature. So what the hell.
		let res = pc.wrapping_add_signed(offset as i16);

		debug!("Fetched relative: {}", res);
		res
	}

	/// Indirect Indexed
	/// Fetches the next byte after opcode, to be used in zero page
	/// The calculation: $(zero page at absolute location ___) + Y
	/// Because zero page is only 256 bytes long, the instruction's address requires 1 byte.
	fn fetch_indirect_y(&self) -> u16 {
		// Read zero-page index address
		let zero_page_indexed_addr: u8 = self.bus.rom.read(self.registers.PC + 1); // read source
		// Get the desired zero-page address
		let addr: u16 = self.bus.ram.read_address(zero_page_indexed_addr as u16); // get the address the source points to

		let res: u16 = self.registers.Y as u16 + addr;
		debug!("Fetched indirect indexed Y: {}", res);
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

/// After each instruction, the bitflags of P register will change.
/// For each bit flag, the change can be: 
/// NONE (no change), MODIFIED (modified), SET (set the bit to 1), CLEARED (set the bit to 0), M6 (memory bit 6), M7 (memory bit 7)
enum P_BitFlag_Operation {
	NONE,
	MODIFIED,
	SET,
	CLEAR,
	M6,
	M7
}