use core::panic;
use log::{debug, error, warn};

use crate::cpu::registers::{Registers, ProcessorStatusRegisterBits};
use crate::cpu::decoder::{OopsCycle, Instructions, AddressingMode, decode_opcode};
use crate::bus::Bus;

use hex::FromHex;

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
		// let p_bits_change = instruction.5;

		//debug!("{:#X}: {:?}\t{:?}\tBytes: {}, Cycles: {}, Oops cycle: {}, P modify: {}", opcode, instr, addrmode, bytes, cycles, oops_cycle, p_bits_change);
		debug!("{:#X}: {:?}\t{:?}\tBytes: {}, Cycles: {}, Oops cycle: {}", opcode, instr, addrmode, bytes, cycles, oops_cycle);

		//The main brains of the CPU. Execute instruction.
		match instr {
			Instructions::LDX => {
				// Load Index X with Memory
				// M -> X
				let fetched_memory = self.fetch_memory(&addrmode);
				self.registers.X = fetched_memory;

				self.modify_p_n(fetched_memory);
				self.modify_p_z(fetched_memory);
			}
			Instructions::LDY => {
				// Load Index Y with Memory
				// M -> Y
				let fetched_memory = self.fetch_memory(&addrmode);
				self.registers.Y = fetched_memory;

				self.modify_p_n(fetched_memory);
				self.modify_p_z(fetched_memory);
			}
			Instructions::LDA => {
				// Load Accumulator with Memory
				// M -> A
				let fetched_memory = self.fetch_memory(&addrmode);
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
			Instructions::SEC => {
				// Set Carry Flag
				self.modify_p_set(ProcessorStatusRegisterBits::CARRY);
			}
			Instructions::CLC => {
				// Clear Carry Flag
				self.modify_p_clear(ProcessorStatusRegisterBits::CARRY);
			}
			Instructions::SED => {
				// Set Decimal Flag
				self.modify_p_set(ProcessorStatusRegisterBits::DECIMAL);
			}
			Instructions::CLD => {
				// Clear Decimal Mode
				self.modify_p_clear(ProcessorStatusRegisterBits::DECIMAL);
			}
			Instructions::CLI => {
				// Clear Interrupt Disable Bit
				self.modify_p_clear(ProcessorStatusRegisterBits::INTERRUPT_DISABLE);
			}
			Instructions::SEI => {
				// Set Interrupt Disable Status
				self.modify_p_set(ProcessorStatusRegisterBits::INTERRUPT_DISABLE);
			}
			Instructions::CLV => {
				// Clear Overflow Flag
				self.modify_p_clear(ProcessorStatusRegisterBits::OVERFLOW);
			}
			Instructions::ADC => {
				// Add Memory to Accumulator with Carry
				// A + M + C -> A, C
				// NOTE: This is the first instruction that actually does 'complex' arithmetic
				// After reading a lot of forums, its actually the most complex thing to emulate, I must understand this

				let fetched_memory = self.fetch_memory(&addrmode);

				let a = self.registers.A;
				let m = fetched_memory;
				let carry: u8 = self.registers.P.get(ProcessorStatusRegisterBits::CARRY) as u8;

				// Carry flag: Only for unsigned. If result is > 255, carry is set.
				// Overflow flag: Only if (Positive+Positive=Negative) or (Negative+Negative=Positive)

				// Perform regular unsigned addition, allowing arithmetic overflow.
				let first_addition = a.overflowing_add(m);
				let second_addition = first_addition.0.overflowing_add(carry);
				let mut result = second_addition.0;

				// Set A register.

				// Check decimal mode, check if CPU is in binary/decimal coded mode
				// TODO: I read that NES doesn't use this mode. Maybe remove it so I don't have any problems?
				if self.registers.P.get(ProcessorStatusRegisterBits::DECIMAL) {
					result = self.decimal_mode(result);
				}
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
			Instructions::STX => {
				// Store Index X in Memory
				// X -> M
				let addr = self.read_instruction_address(addrmode);
				self.bus.memory.write(addr, self.registers.X);
			}
			Instructions::STY => {
				// Store Index Y in Memory
				// Y -> M
				let addr = self.read_instruction_address(addrmode);
				self.bus.memory.write(addr, self.registers.Y);
			}
			Instructions::STA => {
				// Store Accumulator in Memory
				// A -> M
				let addr = self.read_instruction_address(addrmode);
				self.bus.memory.write(addr, self.registers.A);
			}
			Instructions::INX => {
				// Increment Index X by One
				// X + 1 -> X
				self.registers.X = self.registers.X.wrapping_add(1);
				self.modify_p_n(self.registers.X);
				self.modify_p_z(self.registers.X);
			}
			Instructions::INY => {
				// Increment Index Y by One
				// Y + 1 -> Y
				self.registers.Y = self.registers.Y.wrapping_add(1);
				self.modify_p_n(self.registers.Y);
				self.modify_p_z(self.registers.Y);
			}
			Instructions::INC => {
				// Increment Memory by One
				// M + 1 -> M
				let fetched_memory = self.fetch_memory(&addrmode);
				let new_memory = fetched_memory.wrapping_add(1);

				let addr = self.read_instruction_address(addrmode);
				self.bus.memory.write(addr, new_memory);

				self.modify_p_n(new_memory);
				self.modify_p_z(new_memory);
			}
			Instructions::RTI => {
				// Return from Interrupt
				// The status register is pulled with the break flag and bit 5 ignored. Then PC is pulled from the stack.
				// pull SR, pull PC
				// TODO: Complete
			}
			Instructions::CMP => {
				// Compare Memory with Accumulator
				// A - M
				
			}
			_ => {
				error!("Could not execute instruction: {:?}, not implimented, yet", instr);
				panic!();
			}
		}

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

	/// Read immediate from ROM, not from memory!
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
		let res = self.bus.memory.read(addr as u16);
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

	fn fetch_indexed_zero_page(&self, index: u8) -> u8 {
		let instr_addr = self.read_instruction_zero_page_address();
		let addr = instr_addr.wrapping_add(index);
		let res = self.bus.memory.read(addr as u16);
		debug!("Fetched indexed zero page: {:#X}", res);
		res
	}

	// /// Its quite complex, read on internet: https://youtu.be/fWqBmmPQP40?t=721
	// fn fetch_indirect_zero_page_x(&self) -> u8 {
	// 	let instr_addr = self.read_instruction_zero_page_address();
	// 	let addr = instr_addr.wrapping_add(self.registers.X);
	// 	let indexed_addr = self.read_ram_address(addr as u16);
	// 	let res = self.bus.memory.read(indexed_addr);
	// 	debug!("Fetched indirect zero page, x: {:#X}", res);
	// 	res
	// }

	// /// This is NOT like indirect_zero_page_x . Explanation video: https://youtu.be/fWqBmmPQP40?t=751
	// /// Notice we add Y register AFTER and not before calculating indexed address
	// fn fetch_indirect_zero_page_y(&self) -> u8 {
	// 	let addr = self.read_instruction_zero_page_address();
	// 	let indexed_addr = self.read_ram_address(addr as u16) + self.registers.Y as u16;
	// 	let res = self.bus.memory.read(indexed_addr);
	// 	debug!("Fetched indirect zero page, y: {:#X}", res);
	// 	res
	// }

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

	fn read_instruction_absolute_address(&self) -> u16 {
		let lsb = self.bus.rom.read(self.registers.PC + 1) as u16;
		let msb = self.bus.rom.read(self.registers.PC + 2) as u16;
		(msb << 8) | lsb
	}

	fn read_instruction_zero_page_address(&self) -> u8 {
		self.bus.rom.read(self.registers.PC + 1)
	}

	/// $0xFFFA, $0xFFFB
	// fn nmi_interrupt(&self)

	/// $0xFFFC, $0xFFFD
	// fn res_interrupt(&self)

	/// $0xFFFE, $0xFFFF
	// fn irq_interrupt(&self)

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

	fn modify_p_n(&mut self, value: u8) {
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

	/// Convert data from hex (example: 0x0B) to another hex (0x11), but is represented in 'decimal hex' form.
	fn decimal_mode(&self, data: u8) -> u8 {
		let hex_str = data.to_string();
		let decoded = <[u8; 1]>::from_hex(hex_str).expect("Could not convert decimal");
		decoded[0]
	}

	/// Read memory. This can be in ROM (immediate, for example) or in RAM.
	fn fetch_memory(&self, addrmode: &AddressingMode) -> u8 {
		match addrmode {
			//AddressingMode::IMPLIED => 			0, 	// Implied means this instruction doesn't fetch any memory. For now its zero. It won't be used.
			AddressingMode::ABSOLUTE => 		self.fetch_absolute(),
			// AddressingMode::RELATIVE => self.fetch_relative(),
			AddressingMode::IMMEDIATE => 		self.fetch_immediate(),
			AddressingMode::ACCUMULATOR => 		self.fetch_accumulator(),
			// AddressingMode::INDIRECTY => self.fetch_indirect_y(),
			AddressingMode::ZEROPAGE => 		self.fetch_zero_page(),
			AddressingMode::ZEROPAGEX => 		self.fetch_indexed_zero_page(self.registers.X),
			AddressingMode::ZEROPAGEY => 		self.fetch_indexed_zero_page(self.registers.Y),
			_ => {
				error!("The instruction doesn't support addressing mode: {:?}, panic", addrmode);
				panic!();
			}
		}
	}

	/// Extract the address from instruction
	fn read_instruction_address(&self, addrmode: AddressingMode) -> u16 {
		match addrmode {
			AddressingMode::IMMEDIATE => self.read_instruction_immediate_address(),
			AddressingMode::ABSOLUTE => self.read_instruction_absolute_address(),
			AddressingMode::ZEROPAGE => self.read_instruction_zero_page_address() as u16,
			_ => todo!()
		}
	}

	fn read_instruction_immediate_address(&self) -> u16 {
		let res = self.bus.rom.read(self.registers.PC + 1) as u16;
		debug!("Fetched immediate address: {:#X}", res);
		res
	}

}

#[cfg(test)]
mod tests {
    use simple_logger::SimpleLogger;

    use crate::{bus::Bus, program_loader::*, memory::ROM, cpu::registers::ProcessorStatusRegisterBits};

    use super::CPU;

	fn initialize(f: fn(&mut [u8;65_536]) -> u8) -> CPU {
		// Create ROM and load it with any program, for testing.
		let mut rom_memory: [u8; 65_536] = [0;65_536];
		f(&mut rom_memory);  // call f - load program
		let rom: ROM = ROM {
			rom: Box::new(rom_memory)
		};
		let bus = Box::new(Bus::new(rom));
		let cpu = CPU::new(bus);

		cpu
	}

	// NOTE: For each program, the last cpu tick is NOP

	#[test]
	fn stack_test() {
		let mut cpu = initialize(load_program_stack);

		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0x8C);
		cpu.clock_tick();
		assert_eq!(cpu.bus.memory.read(0x1FF), 0x8C);
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0xAB);
		cpu.clock_tick();
		assert_eq!(cpu.bus.memory.read(0x1FE), 0xAB);
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0xAB);
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0x8C);
		assert_eq!(cpu.registers.S, 0xFF);
		cpu.clock_tick();
		assert_eq!(cpu.registers.S, 0x00);
		cpu.clock_tick();
	}

	#[test]
	fn lda_test() {
		let mut cpu = initialize(load_program_lda);

		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0xFF);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::NEGATIVE), true);
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::ZERO), true);
		cpu.clock_tick();
	}

	#[test]
	fn adc_test() {
		let mut cpu = initialize(load_program_adc);

		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::DECIMAL), false);
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0x09);
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::CARRY), false);
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0x0B);
		
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::DECIMAL), true);
		cpu.clock_tick();
		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0x11);

		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::DECIMAL), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::CARRY), false);
		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::CARRY), true);
		assert_eq!(cpu.registers.A, 0x80);

		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::CARRY), false);
		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::OVERFLOW), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::CARRY), true);
		assert_eq!(cpu.registers.A, 0x7F);


		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::OVERFLOW), false);
		cpu.clock_tick();
		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::OVERFLOW), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::NEGATIVE), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::CARRY), false);
		assert_eq!(cpu.registers.A, 0x80);

		cpu.clock_tick();
	}

	#[test]
	fn test_absolute_store() {
		let mut cpu = initialize(load_program_absolute_store);

		cpu.clock_tick();
		cpu.clock_tick();
		cpu.clock_tick();

		assert_eq!(cpu.bus.memory.read(0x2000), 0);
		cpu.clock_tick();
		assert_eq!(cpu.bus.memory.read(0x2000), 0xAB);

		assert_eq!(cpu.bus.memory.read(0x2001), 0);
		cpu.clock_tick();
		assert_eq!(cpu.bus.memory.read(0x2001), 0xAB);
	}

	#[test]
	fn test_index_increment() {
		let mut cpu = initialize(load_program_index_increment);

		cpu.clock_tick();
		assert_eq!(cpu.registers.X, 0xFE);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::NEGATIVE), true);
		cpu.clock_tick();
		assert_eq!(cpu.registers.X, 0xFF);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::NEGATIVE), true);
		cpu.clock_tick();
		assert_eq!(cpu.registers.X, 0x00);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::NEGATIVE), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::ZERO), true);

		cpu.clock_tick();
	}

	#[test]
	fn test_zeropage_store_load_and_memory_increment() {
		let mut cpu = initialize(load_program_zeropage_store_load_and_memory_increment);

		cpu.clock_tick();
		assert_eq!(cpu.registers.X, 0xFE);

		cpu.clock_tick();
		assert_eq!(cpu.bus.memory.read(0x0A), 0xFE);

		cpu.clock_tick();
		assert_eq!(cpu.bus.memory.read(0x0A), 0xFF);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::ZERO), false);
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::ZERO), true);
		assert_eq!(cpu.bus.memory.read(0x0A), 0x00);

		cpu.clock_tick();
	}

	#[test]
	fn test_zeropage_x() {
		SimpleLogger::new().init().unwrap();

		let mut cpu = initialize(load_program_zeropage_x);

		cpu.clock_tick();
		cpu.clock_tick();
		cpu.clock_tick();

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.bus.memory.read(0x0A), 0xFE);
		assert_ne!(cpu.registers.A, 0xFE);
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0xFE);

		cpu.clock_tick();
		assert_eq!(cpu.registers.X, 0x0B);
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0xFC);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::CARRY), true);

		cpu.clock_tick();
	}

}