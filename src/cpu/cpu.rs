use core::panic;
use log::{debug, error, warn};

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
				// M -> Y
				self.registers.Y = fetched_memory;

				self.modify_p_n(fetched_memory);
				self.modify_p_z(fetched_memory);
			}
			Instructions::LDA => {
				// Load Accumulator with Memory
				// M -> A
				self.registers.A = fetched_memory;

				self.modify_p_n(fetched_memory);
				self.modify_p_z(fetched_memory);
			}
			Instructions::PHA => {
				// Push Accumulator on Stack
				// push A
				self.push_stack(self.registers.A);
			}
			Instructions::NOP => {
				// No Operation
			}
			Instructions::PLA => {
				// Pull Accumulator from Stack
				// pull A
				let fetched_memory = self.pop_stack();
				self.registers.A = fetched_memory;

				self.modify_p_n(fetched_memory);
				self.modify_p_z(fetched_memory);
			}
			Instructions::CLC => {
				// Clear Carry Flag
				self.modify_p_clear(ProcessorStatusRegisterBits::CARRY);
			}
			Instructions::CLD => {
				// Clear Decimal Mode
				self.modify_p_clear(ProcessorStatusRegisterBits::DECIMAL);
			}
			Instructions::CLI => {
				// Clear Interrupt Disable Bit
				self.modify_p_clear(ProcessorStatusRegisterBits::INTERRUPT_DISABLE);
			}
			Instructions::CLV => {
				// Clear Overflow Flag
				self.modify_p_clear(ProcessorStatusRegisterBits::OVERFLOW);
			}
			Instructions::SEC => {
				// Set Carry Flag
				self.modify_p_set(ProcessorStatusRegisterBits::CARRY);
			}
			Instructions::SED => {
				// Set Decimal Flag
				self.modify_p_set(ProcessorStatusRegisterBits::DECIMAL);
			}
			Instructions::ADC => {
				// Add Memory to Accumulator with Carry
				// A + M + C -> A, C
				// NOTE: This is the first instruction that actually does 'complex' arithmetic
				// After reading a lot of forums, its actually the most complex thing to emulate, I must understand this

				let a = self.registers.A;
				let m = fetched_memory;
				let carry: u8 = self.registers.P.get(ProcessorStatusRegisterBits::CARRY) as u8;

				// Carry flag: Only for unsigned. If result is > 255, carry is set.
				// Overflow flag: Only if (Positive+Positive=Negative) or (Negative+Negative=Positive)

				// Perform regular unsigned addition, allowing arithmetic overflow.
				let first_addition = a.overflowing_add(m);
				let second_addition = first_addition.0.overflowing_add(carry);
				let result = second_addition.0;

				// Set A register.
				self.registers.A = result;

				// Set carry accordingly.
				let new_carry = first_addition.1 || second_addition.1;

				// Set overflow accordingly.
				let is_a_negative = (a >> 7) == 1;
				let is_m_negative = (m >> 7) == 1;
				let is_result_negative = (result >> 7) == 1;
				let new_overflow = 
					(is_a_negative 				&& is_m_negative 			&& is_result_negative == false 	) ||
					(is_a_negative == false 	&& is_m_negative == false 	&& is_result_negative 			);
				
				self.modify_p_n(self.registers.A);
				self.modify_p_z(self.registers.A);
				self.modify_p_c(new_carry);
				self.modify_p_v(new_overflow);
			}
			Instructions::STA => {
				// Store Accumulator in Memory
				// A -> M

				//TODO: Complete

			}
			_ => {
				error!("Could not execute instruction: {:?}, not implimented, yet", instr);
				panic!();
			}
		}

		/*
		http://www.6502.org/tutorials/vflag.html
		When the addition result is 0 to 255, the carry is cleared.
		When the addition result is greater than 255, the carry is set.
		When the subtraction result is 0 to 255, the carry is set.
		When the subtraction result is less than 0, the carry is cleared.
		*/

		// Modify P register.
		// self.modify_p(ProcessorStatusRegisterBits::NEGATIVE, 			p_bits_change.n, fetched_memory);
		// self.modify_p(ProcessorStatusRegisterBits::ZERO, 				p_bits_change.z, fetched_memory);
		// self.modify_p(ProcessorStatusRegisterBits::CARRY, 				p_bits_change.c, fetched_memory);
		// self.modify_p(ProcessorStatusRegisterBits::INTERRUPT_DISABLE, 	p_bits_change.i, fetched_memory);
		// self.modify_p(ProcessorStatusRegisterBits::DECIMAL, 			p_bits_change.d, fetched_memory);
		// self.modify_p(ProcessorStatusRegisterBits::OVERFLOW, 			p_bits_change.v, fetched_memory);

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
		let res = self.bus.memory.read(abs_addr);
		debug!("Fetched absolute: {:#X}", res);
		res
	}

	fn fetch_zero_page(&self) -> u8 {
		let addr = self.read_instruction_zero_page_address();
		let res = self.bus.memory.read(addr);
		debug!("Fetched from zero page: {:#X}", res);
		res
	}

	fn fetch_absolute_x(&self) -> u8 {
		let addr = self.read_instruction_absolute_address() + self.registers.X as u16;
		let res = self.bus.memory.read(addr);
		debug!("Fetched absolute,X: {:#X}", res);
		res
	}

	fn fetch_absolute_y(&self) -> u8 {
		let addr = self.read_instruction_absolute_address() + self.registers.Y as u16;
		let res = self.bus.memory.read(addr);
		debug!("Fetched absolute,Y: {:#X}", res);
		res
	}

	fn fetch_zero_page_x(&self) -> u8 {
		let addr = self.read_instruction_zero_page_address() + self.registers.X as u16;
		let res = self.bus.memory.read(addr);
		debug!("Fetched zero page, x: {:#X}", res);
		res
	}

	fn fetch_zero_page_y(&self) -> u8 {
		let addr = self.read_instruction_zero_page_address() + self.registers.Y as u16;
		let res = self.bus.memory.read(addr);
		debug!("Fetched zero page, x: {:#X}", res);
		res
	}

	/// Its quite complex, read on internet: https://youtu.be/fWqBmmPQP40?t=721
	fn fetch_indirect_zero_page_x(&self) -> u8 {
		let addr = self.read_instruction_zero_page_address() + self.registers.X as u16;
		let indexed_addr = self.read_ram_address(addr);
		let res = self.bus.memory.read(indexed_addr);
		debug!("Fetched indirect zero page, x: {:#X}", res);
		res
	}

	/// This is NOT like indirect_zero_page_x . Explanation video: https://youtu.be/fWqBmmPQP40?t=751
	/// Notice we add Y register AFTER and not before calculating indexed address
	fn fetch_indirect_zero_page_y(&self) -> u8 {
		let addr = self.read_instruction_zero_page_address();
		let indexed_addr = self.read_ram_address(addr) + self.registers.Y as u16;
		let res = self.bus.memory.read(indexed_addr);
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
		let msb = self.bus.memory.read(addr) as u16;
		let lsb = self.bus.memory.read(addr + 1) as u16;
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

	/// $0xFFFA, $0xFFFB
	// fn nmi_interrupt(&self) {
	// 	//TODO: Complete
	// }

	/// $0xFFFC, $0xFFFD
	// fn res_interrupt(&self) {

	/// $0xFFFE, $0xFFFF
	// fn irq_interrupt(&self) {
	// 	//TODO: Complete
	// }

	
	// 	//TODO: Complete
	// }

	fn push_stack(&mut self, data: u8) {
		self.bus.memory.write(0x100 + self.registers.S as u16, data);
		self.registers.S -= 1;
		debug!("Pushed to stack: \t{:#X}", data);
	}

	fn pop_stack(&mut self) -> u8 {
		if self.registers.S == 0xFF {
			warn!("Stack pop: stack pointer is at beginning, overflowing stack pointer");
		}
		let head_addr: u16 = 0x100 + (self.registers.S as u16) + 1;  // we add 1 before the current SP points to get the head (the stack is down going)
		let res = self.bus.memory.read(head_addr);
		self.registers.S = self.registers.S.wrapping_add(1);  // NOTE: We allow the programmer to overflow SP.
		//self.registers.S += 1;
		debug!("Poped stack: \t{:#X}", res);
		res
	}

	fn  modify_p_n(&mut self, value: u8) {
		// If last bit (7) is 1, its negative
		self.registers.P.set(ProcessorStatusRegisterBits::NEGATIVE, (value >> 7) == 1);
	}

	fn modify_p_z(&mut self, value: u8) {
		// If value is 0, zero flag is 1
		self.registers.P.set(ProcessorStatusRegisterBits::ZERO, value == 0); 
	}

	fn modify_p_c(&mut self, carry: bool) {
		// If carry detected, set carry flag to 1
		self.registers.P.set(ProcessorStatusRegisterBits::CARRY, carry);
	}

	fn modify_p_set(&mut self, bit: ProcessorStatusRegisterBits) {
		self.registers.P.set(bit, true);
	}

	fn modify_p_clear(&mut self, bit: ProcessorStatusRegisterBits) {
		self.registers.P.set(bit, false);
	}

	fn modify_p_v(&mut self, overflow: bool) {
		// It's complex, read online, I let the instructions handle the logic
		self.registers.P.set(ProcessorStatusRegisterBits::OVERFLOW, overflow);
	}

	fn decimal_mode(&self, data: u8) {
		// TODO: Complete.
		// Convert data from hex (example: 0x0B) to another hex (0x11), but is represented in 'decimal hex' form.
	}

	// fn arithmetic_add_2(&mut self, a: u8, b: u8) -> (u8, bool) {
	// 	self.arithmetic_add_3(a, b, 0)
	// }

	// /// Does a+b+c. If overflow occured in (a+b=d) or (d+c), returns true.
	// /// I know, arithmetic overflow, why I set CARRY flag?
	// /// Read here: http://www.6502.org/tutorials/vflag.html
	// /// The overflow bitflag is not what you think.
	// fn arithmetic_add_3(&mut self, a: u8, b: u8, c: u8) -> (u8, bool) {
	// 	let res = a.overflowing_add(b);
	// 	let res2 = res.0.overflowing_add(c);
	// 	let overflow = res.1 || res2.1;

	// 	let sum = res2.0;

	// 	if (A^res) & ()

	// 	(sum, overflow)
	// }
}

mod test {
    use log::info;
    use simple_logger::SimpleLogger;

    use crate::{bus::Bus, program_loader::*, memory::ROM};

    use super::CPU;

	use std::sync::Once;

	static INIT: Once = Once::new();

	static mut ROM: [u8; 65_536] = [0;65_536];

	fn initialize() {
		INIT.call_once(|| {
			SimpleLogger::new().init().unwrap();

			unsafe {
				// Create ROM and load it with simple program.
				//let mut rom_memory: [u8; 65_536] = [0;65_536];
				let assembly_lines_amount = load_program_overflow_2(&mut ROM);
				let rom: ROM = ROM {
					rom: Box::new(ROM)
				};
				// Create CPU.
				let bus = Box::new(Bus::new(rom));
				let mut cpu = CPU::new(bus);
			
				// Execute clocks.
				for _ in 0..assembly_lines_amount {
					cpu.clock_tick();
				}
			}
		});
	}

	#[test]
	fn foo_test() {
		initialize();
		unsafe {
			assert_eq!(ROM[0], 0x18);
			load_program_reset_sp(&mut ROM);
			assert_eq!(ROM[0], 0x68);
		}
	}
}