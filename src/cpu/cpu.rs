use core::panic;
use log::{debug, error};

use crate::cpu::registers::{Registers, ProcessorStatusRegisterBits};
use crate::cpu::decoder::{OopsCycle, Instructions, AddressingMode, decode_opcode, ProcessorStatusRegisterBitChanges};
use crate::bus::Bus;

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
		let mut registers: Registers = Registers::default();
		registers.S = 0xFF; //TODO: Remove. The original NES does not initialize the stack register; Its random at startup. But I need this to debug my programs for now.
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
		let instruction = decode_opcode(opcode);

		let instr = instruction.0;
		let addrmode = instruction.1;
		let bytes = instruction.2;
		let cycles = instruction.3;
		let oops_cycle = instruction.4;
		let p_bits_change = instruction.5;

		debug!("{:#X}: {:?}\t{:?}\tBytes: {}, Cycles: {}, Oops cycle: {}, P modify: {}", opcode, instr, addrmode, bytes, cycles, oops_cycle, p_bits_change);

		fn panic_addressing_mode_unsupported(addrmode: AddressingMode) {
			error!("The instruction doesn't support addressing mode: {:?}, panic", addrmode);
			panic!();
		}

		// Here we allowed to cast u8 to u16 because its the Address bus. The CPU only supports address bus of 16 signals (bits).
		let fetched_memory: u8 = match addrmode {
			AddressingMode::IMPLIED => 			0, 	// Implied means this instruction doesn't fetch any memory. For now its zero. It won't be used.
			AddressingMode::ABSOLUTE => 		self.fetch_absolute(),
			// AddressingMode::RELATIVE => self.fetch_relative(),
			AddressingMode::IMMEDIATE => 		self.fetch_immediate(),
			AddressingMode::ACCUMULATOR => 		self.fetch_accumulator(),
			// AddressingMode::INDIRECTY => self.fetch_indirect_y(),
			AddressingMode::ZEROPAGE => 		self.fetch_zero_page(),
			AddressingMode::ZEROPAGEX => 		self.fetch_zero_page_x(),
			AddressingMode::ZEROPAGEY => 		self.fetch_zero_page_y(),
			
			_ => {
				panic_addressing_mode_unsupported(addrmode);
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
			Instructions::PHA => {
				// Push Accumulator on Stack
				self.push_stack(self.registers.A);
			}

			_ => {
				error!("Could not execute instruction: {:?}, not implimented, yet", instr);
				panic!();
			}
		}

		// Modify P register.
		self.modify_p(ProcessorStatusRegisterBits::NEGATIVE, 			p_bits_change.n, fetched_memory);
		self.modify_p(ProcessorStatusRegisterBits::ZERO, 				p_bits_change.z, fetched_memory);
		self.modify_p(ProcessorStatusRegisterBits::CARRY, 				p_bits_change.c, fetched_memory);
		self.modify_p(ProcessorStatusRegisterBits::INTERRUPT_DISABLE, 	p_bits_change.i, fetched_memory);
		self.modify_p(ProcessorStatusRegisterBits::DECIMAL, 			p_bits_change.d, fetched_memory);
		self.modify_p(ProcessorStatusRegisterBits::OVERFLOW, 			p_bits_change.v, fetched_memory);

		// Increment PC by amount of bytes needed for the instruction, other than opcode (which is 1 byte).
		// We do this at the end of the execution, because we need to access the PC (for the current instruction) before we increment it.
		// For example, when we have LDA, we load A with immediate memory at the next byte of PC. So we access PC + 1.
		self.registers.PC += bytes as u16;

		self.cycles += cycles as u64;

		match oops_cycle {
			OopsCycle::NONE => { 
				// don't change amount of cycles.
			},
			OopsCycle::PageBoundryCrossed => { 
				//TODO: Impliment. For now, I don't change amount of cycles.

				//add 1 to cycles if page boundary is crossed
			},
			OopsCycle::BranchOccursOn => {
				//TODO: Impliment. For now, I don't change amount of cycles.

				//add 1 to cycles if branch occurs on same page
				//add 2 to cycles if branch occurs to different page
			}
		}
	}

	//TODO: Optimize all fetch functions as inline?

	fn fetch_immediate(&self) -> u8 {
		let res = self.bus.rom.read(self.registers.PC + 1);
		debug!("Fetched immediate: {:#X}", res);
		res
	}

	fn fetch_absolute(&self) -> u8 {
		let abs_addr = self.read_instruction_absolute_address();
		let res = self.bus.ram.read(abs_addr);
		debug!("Fetched absolute: {:#X}", res);
		res
	}

	fn fetch_zero_page(&self) -> u8 {
		let addr = self.read_instruction_zero_page_address();
		let res = self.bus.ram.read(addr);
		debug!("Fetched from zero page: {:#X}", res);
		res
	}

	fn fetch_absolute_x(&self) -> u8 {
		let addr = self.read_instruction_absolute_address() + self.registers.X as u16;
		let res = self.bus.ram.read(addr);
		debug!("Fetched absolute,X: {:#X}", res);
		res
	}

	fn fetch_absolute_y(&self) -> u8 {
		let addr = self.read_instruction_absolute_address() + self.registers.Y as u16;
		let res = self.bus.ram.read(addr);
		debug!("Fetched absolute,Y: {:#X}", res);
		res
	}

	fn fetch_zero_page_x(&self) -> u8 {
		let addr = self.read_instruction_zero_page_address() + self.registers.X as u16;
		let res = self.bus.ram.read(addr);
		debug!("Fetched zero page, x: {:#X}", res);
		res
	}

	fn fetch_zero_page_y(&self) -> u8 {
		let addr = self.read_instruction_zero_page_address() + self.registers.Y as u16;
		let res = self.bus.ram.read(addr);
		debug!("Fetched zero page, x: {:#X}", res);
		res
	}

	/// Its quite complex, read on internet: https://youtu.be/fWqBmmPQP40?t=721
	fn fetch_indirect_zero_page_x(&self) -> u8 {
		let addr = self.read_instruction_zero_page_address() + self.registers.X as u16;
		let indexed_addr = self.read_ram_address(addr);
		let res = self.bus.ram.read(indexed_addr);
		debug!("Fetched indirect zero page, x: {:#X}", res);
		res
	}

	/// This is NOT like indirect_zero_page_x . Explanation video: https://youtu.be/fWqBmmPQP40?t=751
	/// Notice we add Y register AFTER and not before calculating indexed address
	fn fetch_indirect_zero_page_y(&self) -> u8 {
		let addr = self.read_instruction_zero_page_address();
		let indexed_addr = self.read_ram_address(addr) + self.registers.Y as u16;
		let res = self.bus.ram.read(indexed_addr);
		debug!("Fetched indirect zero page, y: {:#X}", res);
		res
	}

	fn fetch_accumulator(&self) -> u8 {
		let res = self.registers.A;
		debug!("Fetched accumulator: {}", res);
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

	fn read_ram_address(&self, addr: u16) -> u16 {
		let msb = self.bus.ram.read(addr) as u16;
		let lsb = self.bus.ram.read(addr + 1) as u16;
		(msb << 8) | lsb
	}

	fn read_rom_address(&self, addr: u16) -> u16 {
		let msb = self.bus.rom.read(addr) as u16;
		let lsb = self.bus.rom.read(addr + 1) as u16;
		(msb << 8) | lsb
	}

	fn read_instruction_absolute_address(&self) -> u16 {
		let msb = self.bus.rom.read(self.registers.PC +1) as u16;
		let lsb = self.bus.rom.read(self.registers.PC +2) as u16;
		(msb << 8) | lsb
	}

	fn read_instruction_zero_page_address(&self) -> u16 {
		self.bus.rom.read(self.registers.PC + 1) as u16
	}

	fn modify_p(&mut self, bit: ProcessorStatusRegisterBits, job: ProcessorStatusRegisterBitChanges, fetched_memory: u8) {
		//TODO: Complete
		match job {
			ProcessorStatusRegisterBitChanges::CLEARED => { 
				self.registers.P.set(bit, false) 
			},
			ProcessorStatusRegisterBitChanges::SET => { 
				self.registers.P.set(bit, true) 
			},
			ProcessorStatusRegisterBitChanges::MODIFIED => { 
				match bit {
					ProcessorStatusRegisterBits::ZERO => { 
						// If memory is 0, zero flag is 1
						self.registers.P.set(bit, fetched_memory == 0); 
					}
					ProcessorStatusRegisterBits::NEGATIVE => { 
						// If last bit (7) is 1, its negative
						self.registers.P.set(bit, (fetched_memory >> 7) == 1);
					}
					_ => {
						panic!("The P bit to modify is unsupported: {:?}, yet", bit);
					}
				}
			},
			ProcessorStatusRegisterBitChanges::M6 => {

			},
			ProcessorStatusRegisterBitChanges::M7 => {

			}
			ProcessorStatusRegisterBitChanges::FromStack => {

			}
			ProcessorStatusRegisterBitChanges::NotModified => {
				//do nothing
			}
		};
	}

	// fn nmi_interrupt(&self) {
	// 	//TODO: Complete
	// }

	// fn irq_interrupt(&self) {
	// 	//TODO: Complete
	// }

	// fn reset_interrupt(&self) {
	// 	//TODO: Complete
	// }

	fn push_stack(&mut self, data: u8) {
		self.bus.ram.write(0x100 + self.registers.S as u16, data);
		self.registers.S -= 1;
		debug!("Pushed to stack: \t{:#X}", data);
	}

	fn pop_stack(&mut self) -> u8 {
		let res = self.bus.ram.read(0x100 + self.registers.S as u16);
		self.registers.S += 1;
		debug!("Poped stack: \t{:#X}", res);
		res
	}
}
