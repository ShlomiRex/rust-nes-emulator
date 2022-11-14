use core::panic;
use log::{debug, error, warn};

use crate::cpu::registers::{Registers, ProcessorStatusRegisterBits};
use crate::cpu::decoder::{OopsCycle, Instructions, AddressingMode, decode_opcode};
use crate::bus::Bus;

use hex::FromHex;

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
	/// Original NES CPU needs multiple cycles to execute instruction.
	/// Emulation does not do that; Its much simpler to do everything at once, and emulate the cycles.
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

		debug!("{:#X}: {:?}\t{:?}\tBytes: {}, Cycles: {}, Oops cycle: {}", opcode, instr, addrmode, bytes, cycles, oops_cycle);

		//The main brains of the CPU. Execute instruction.
		match instr {
			Instructions::LDX => {
				// Load Index X with Memory
				// M -> X
				let fetched_memory = self.fetch_memory(&addrmode);
				self.registers.X = fetched_memory;

				self.registers.P.modify_n(fetched_memory);
				self.registers.P.modify_z(fetched_memory);
			}
			Instructions::LDY => {
				// Load Index Y with Memory
				// M -> Y
				let fetched_memory = self.fetch_memory(&addrmode);
				self.registers.Y = fetched_memory;

				self.registers.P.modify_n(fetched_memory);
				self.registers.P.modify_z(fetched_memory);
			}
			Instructions::LDA => {
				// Load Accumulator with Memory
				// M -> A
				let fetched_memory = self.fetch_memory(&addrmode);
				self.registers.A = fetched_memory;

				self.registers.P.modify_n(fetched_memory);
				self.registers.P.modify_z(fetched_memory);
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

				self.registers.P.modify_n(fetched_memory);
				self.registers.P.modify_z(fetched_memory);
			}
			Instructions::SEC => {
				// Set Carry Flag
				self.registers.P.set(ProcessorStatusRegisterBits::CARRY, true);
			}
			Instructions::CLC => {
				// Clear Carry Flag
				self.registers.P.set(ProcessorStatusRegisterBits::CARRY, false);
			}
			Instructions::SED => {
				// Set Decimal Flag
				self.registers.P.set(ProcessorStatusRegisterBits::DECIMAL, true);
			}
			Instructions::CLD => {
				// Clear Decimal Mode
				self.registers.P.set(ProcessorStatusRegisterBits::DECIMAL, false);
			}
			Instructions::SEI => {
				// Set Interrupt Disable Status
				self.registers.P.set(ProcessorStatusRegisterBits::INTERRUPT_DISABLE, true);
			}
			Instructions::CLI => {
				// Clear Interrupt Disable Bit
				self.registers.P.set(ProcessorStatusRegisterBits::INTERRUPT_DISABLE, false);
			}
			Instructions::CLV => {
				// Clear Overflow Flag
				self.registers.P.set(ProcessorStatusRegisterBits::OVERFLOW, false);
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
				
				self.registers.P.modify_n(self.registers.A);
				self.registers.P.modify_z(self.registers.A);
				self.registers.P.set(ProcessorStatusRegisterBits::CARRY, new_carry);
				self.registers.P.set(ProcessorStatusRegisterBits::OVERFLOW, new_overflow);
			}
			Instructions::STX => {
				// Store Index X in Memory
				// X -> M
				let addr = self.fetch_instruction_address(addrmode);
				self.bus.memory.write(addr, self.registers.X);
			}
			Instructions::STY => {
				// Store Index Y in Memory
				// Y -> M
				let addr = self.fetch_instruction_address(addrmode);
				self.bus.memory.write(addr, self.registers.Y);
			}
			Instructions::STA => {
				// Store Accumulator in Memory
				// A -> M
				let addr = self.fetch_instruction_address(addrmode);
				self.bus.memory.write(addr, self.registers.A);
			}
			Instructions::INX => {
				// Increment Index X by One
				// X + 1 -> X
				self.registers.X = self.registers.X.wrapping_add(1);
				self.registers.P.modify_n(self.registers.X);
				self.registers.P.modify_z(self.registers.X);
			}
			Instructions::INY => {
				// Increment Index Y by One
				// Y + 1 -> Y
				self.registers.Y = self.registers.Y.wrapping_add(1);
				self.registers.P.modify_n(self.registers.Y);
				self.registers.P.modify_z(self.registers.Y);
			}
			Instructions::INC => {
				// Increment Memory by One
				// M + 1 -> M
				let fetched_memory = self.fetch_memory(&addrmode);
				let new_memory = fetched_memory.wrapping_add(1);

				let addr = self.fetch_instruction_address(addrmode);
				self.bus.memory.write(addr, new_memory);

				self.registers.P.modify_n(new_memory);
				self.registers.P.modify_z(new_memory);
			}
			Instructions::JMP => {
				// Jump to New Location
				// (PC+1) -> PCL
				// (PC+2) -> PCH
				let addr = self.fetch_instruction_address(addrmode);
				self.registers.PC = addr;
			}
			Instructions::JSR => {
				// Jump to New Location Saving Return Address

				// push (PC+2),
				// (PC+1) -> PCL
				// (PC+2) -> PCH

				// What order of bytes to push?
				// After a lot of googling: https://stackoverflow.com/a/63886154
				// Basically push the PC like so: "...You need to push the high byte first, and then the low byte."

				// Push PC onto stack (return address)
				// NOTE: I push the 3rd byte of the instruction (PC + 2). Why not PC+3 (next instruction)?
				// Idk, but its important to emulate this exactly, because some games use this feature.
				let pc_msb = (self.registers.PC.wrapping_add(2) >> 8) as u8;
				let pc_lsb = (self.registers.PC.wrapping_add(2)) as u8;
				self.push_stack(pc_msb);
				self.push_stack(pc_lsb);

				// Jump to the address operand
				let addr = self.fetch_instruction_address(addrmode);
				self.registers.PC = addr;
			}
			Instructions::CMP => {
				// Compare Memory with Accumulator
				// A - M

				self.exec_cmp(addrmode, self.registers.A);
			}
			Instructions::CPX => {
				// Compare Memory and Index X
				// X - M

				self.exec_cmp(addrmode, self.registers.X);
			}
			Instructions::CPY => {
				// Compare Memory and Index Y
				// Y - M

				self.exec_cmp(addrmode, self.registers.Y);
			}
			Instructions::BIT => {
				// Test Bits in Memory with Accumulator

				// bits 7 and 6 of operand are transfered to bit 7 and 6 of SR (N,V);
				// the zero-flag is set to the result of operand AND accumulator.

				// A AND M, M7 -> N, M6 -> V

				todo!();
			}
			_ => {
				panic!("Could not execute instruction: {:?}, not implimented, yet", instr);
			}
		}

		// Increment PC by amount of bytes needed for the instruction, other than opcode (which is 1 byte).
		// We do this at the end of the execution, because we need to access the PC (for the current instruction) before we increment it.
		// For example, when we have LDA, we load A with immediate memory at the next byte of PC. So we access PC + 1.
		// We also don't want to change PC if the instruction changes the PC.
		match instr {
			Instructions::JMP => (),
			Instructions::JSR => (),
			_ => {self.registers.PC += bytes as u16;}
		}

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

	// /// Relative addressing is PC + offset.
	// /// The offset is the next byte after opcode.
	// /// So we fetch the next byte (after opcode) and add it with PC.
	// /// IMPORTANT: The offset is SIGNED. Which means, the offset can be -128 to 127.
	// fn fetch_relative(&self) -> u16 {
	// 	let pc = self.registers.PC;
	// 	let offset = self.bus.rom.read(self.registers.PC + 1) as i8;
	// 	// Here we need a way to add 'u16' type with 'i8' type.
	// 	// IMPORTANT NOTE: We need the "mixed_integer_ops" feature, which is in nightly rust.
	// 	// Its very complex to do this manually, without this feature. So what the hell.
	// 	let res = pc.wrapping_add_signed(offset as i16);
	// 	debug!("Fetched relative: {}", res);
	// 	res
	// }

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

	/// Convert data from hex (example: 0x0B) to another hex (0x11), but is represented in 'decimal hex' form.
	fn decimal_mode(&self, data: u8) -> u8 {
		let hex_str = data.to_string();
		let decoded = <[u8; 1]>::from_hex(hex_str).expect("Could not convert decimal");
		decoded[0]
	}

	fn fetch_absolute_indexed(&self, index: u8) -> u8 {
		let addr = self.read_instruction_absolute_indexed_address(index);
		self.bus.memory.read(addr)
	}

	fn fetch_zero_page_indexed(&self, index: u8) -> u8 {
		let instr_addr = self.read_instruction_zero_page_address();
		let addr = instr_addr.wrapping_add(index);
		self.bus.memory.read(addr as u16)
	}

	/// Read memory. This can be in ROM (immediate, for example) or in RAM (absolute, for example).
	/// All load instructions use this.
	fn fetch_memory(&self, addrmode: &AddressingMode) -> u8 {
		match addrmode {
			AddressingMode::IMPLIED => {
				panic!("Instruction with implied addressing mode should never ask to fetch memory.");
			}
			AddressingMode::IMMEDIATE => {
				let addr = self.registers.PC + 1;
				let res = self.bus.rom.read(addr);
				debug!("Fetched immediate: {:#X}", res);
				res
			}
			AddressingMode::ACCUMULATOR => {
				let res = self.registers.A;
				debug!("Fetched accumulator: {}", res);
				res
			},
			AddressingMode::ZEROPAGE => {
				let addr = self.read_instruction_zero_page_address();
				let res = self.bus.memory.read(addr as u16);
				debug!("Fetched from zero page: {:#X}", res);
				res
			},
			AddressingMode::ZEROPAGEX => {
				let res = self.fetch_zero_page_indexed(self.registers.X);
				debug!("Fetched zeropage,x: {:#X}", res);
				res
			}
			AddressingMode::ZEROPAGEY => {
				let res = self.fetch_zero_page_indexed(self.registers.Y);
				debug!("Fetched zeropage,y: {:#X}", res);
				res
			},
			AddressingMode::ABSOLUTE => {
				let res = self.fetch_absolute_indexed(0);
				debug!("Fetched absolute: {:#X}", res);
				res
			},
			AddressingMode::ABSOLUTEX => {
				let res = self.fetch_absolute_indexed(self.registers.X);
				debug!("Fetched absolute,X: {:#X}", res);
				res
			}
			AddressingMode::ABSOLUTEY => {
				let res = self.fetch_absolute_indexed(self.registers.Y);
				debug!("Fetched absolute,Y: {:#X}", res);
				res
			}
			_ => {
				error!("The instruction doesn't support addressing mode: {:?}, panic", addrmode);
				panic!();
			}
		}
	}

	/// Extract the address from instruction. This function will access ROM and RAM, aswell as indirect addressing.
	/// All store instructions use this.
	fn fetch_instruction_address(&self, addrmode: AddressingMode) -> u16 {
		match addrmode {
			AddressingMode::IMMEDIATE => {
				let res = self.bus.rom.read(self.registers.PC + 1) as u16;
				debug!("Fetched immediate address: {:#X}", res);
				res
			}
			AddressingMode::ABSOLUTE => 	self.read_instruction_absolute_address(),
			AddressingMode::ZEROPAGE => 	self.read_instruction_zero_page_address() as u16,
			AddressingMode::INDIRECT => 	self.read_instruction_indirect_address(),
			AddressingMode::ABSOLUTEX => 	self.read_instruction_absolute_indexed_address(self.registers.X),
			AddressingMode::ABSOLUTEY => 	self.read_instruction_absolute_indexed_address(self.registers.Y),
			_ => todo!()
		}
	}

	/// Reads address stored in ROM at the current PC.
	fn read_instruction_absolute_address(&self) -> u16 {
		let lsb = self.bus.rom.read(self.registers.PC + 1) as u16;
		let msb = self.bus.rom.read(self.registers.PC + 2) as u16;
		(msb << 8) | lsb
	}

	/// Adds absolute address with index.
	fn read_instruction_absolute_indexed_address(&self, index: u8) -> u16 {
		self.read_instruction_absolute_address() + (index as u16)
	}

	/// Reads zero-page address stored in ROM at the current PC.
	fn read_instruction_zero_page_address(&self) -> u8 {
		self.bus.rom.read(self.registers.PC + 1)
	}

	/// Returns address stored in memory, from the absolute address in ROM, at the current PC.
	fn read_instruction_indirect_address(&self) -> u16 {
		let indirect_addr = self.read_instruction_absolute_address();
		let lsb = self.bus.memory.read(indirect_addr) as u16;
		let msb = self.bus.memory.read(indirect_addr + 1) as u16;
		(msb << 8) | lsb
	}

	/// Execute cmp instruction.
	/// Possible instructions: CMP (A register), CPX (X register), CPY (Y register).
	fn exec_cmp(&mut self, addrmode: AddressingMode, register: u8) {
		/*
		Link: http://www.6502.org/tutorials/compare_instructions.html
		Compare Results | N | Z | C
		---------------------------
		A, X, or Y < M  | * | 0 | 0
		A, X, or Y = M  | 0 | 1 | 1
		A, X, or Y > M  | * | 0 | 1

		*The N flag will be bit 7 of A, X, or Y - Memory
		*/
		let fetched_memory = self.fetch_memory(&addrmode);
				
		let sub = register.wrapping_sub(fetched_memory);
		let last_bit = (sub >> 7) == 1;

		let (new_n, new_z, new_c) = 
			if register < fetched_memory {
				(last_bit, false, false)
			} else if register == fetched_memory {
				(false, true, true)
			} else {
				(last_bit, false, true)
			};

		self.registers.P.set(ProcessorStatusRegisterBits::NEGATIVE, new_n);
		self.registers.P.set(ProcessorStatusRegisterBits::ZERO, new_z);
		self.registers.P.set(ProcessorStatusRegisterBits::CARRY, new_c);
	}

}

#[cfg(test)]
mod tests {
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

	// NOTE: For each program, the last cpu tick is NOP, except for branch instructions, the last instruction in those is the stored instruction in memory.

	#[test]
	fn test_stack() {
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
	fn test_lda() {
		let mut cpu = initialize(load_program_lda);

		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0xFF);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::NEGATIVE), true);
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::ZERO), true);
		cpu.clock_tick();
	}

	#[test]
	fn test_adc() {
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

	#[test]
	fn test_absolute_indexed() {
		let mut cpu = initialize(load_program_absolute_indexed);

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.bus.memory.read(0xABCD), 0x0A);
		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.Y, 0x0A);

		cpu.clock_tick();
		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0x0A);

		cpu.clock_tick();
	}

	#[test]
	fn test_jmp_absolute() {
		let mut cpu = initialize(load_program_jmp_absolute);

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.bus.memory.read(0x0001), 0xF8); 	// Instruction SED (0xF8) is stored in memory location 0x0001. It's 1 byte long instruction.

		assert_ne!(cpu.registers.PC, 0x0001);
		cpu.clock_tick();
		assert_eq!(cpu.registers.PC, 0x0001);  // PC is at 0x0001

		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::DECIMAL), false);
		// Execute instruction stored in 0x0001
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::DECIMAL), true);
	}

	#[test]
	fn test_jmp_indirect() {
		let mut cpu = initialize(load_program_jmp_indirect);

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.bus.memory.read(0x00AB), 0x05);

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.bus.memory.read(0x00AC), 0xFF);

		cpu.clock_tick();
		assert_eq!(cpu.registers.PC, 0xFF05);
	}

	#[test]
	fn test_cmp() {
		let mut cpu = initialize(load_program_cmp);

		cpu.clock_tick();
		
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::CARRY), false);
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::CARRY), true);

		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::ZERO), false);
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::ZERO), true);

		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::NEGATIVE), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::ZERO), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::CARRY), false);

		cpu.clock_tick(); // LDA 0xAA: N=1, Z=C=0
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::NEGATIVE), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::ZERO), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::CARRY), true);

		cpu.clock_tick(); // LDA 0x00
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::NEGATIVE), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::ZERO), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::CARRY), false);

		cpu.clock_tick();
	}

	#[test]
	fn test_cpx() {
		// cpy is same...
		let mut cpu = initialize(load_program_cpx);

		cpu.clock_tick();
		cpu.clock_tick();

		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::CARRY), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::ZERO), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::NEGATIVE), false);
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::CARRY), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::ZERO), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::NEGATIVE), true);

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::CARRY), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::ZERO), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::NEGATIVE), true);

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::CARRY), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::ZERO), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::NEGATIVE), false);

		cpu.clock_tick();
	}

	#[test]
	fn test_jsr() {
		let mut cpu = initialize(load_program_jsr);

		let pc_before = cpu.registers.PC;

		assert_ne!(cpu.registers.PC, 0x0A0B);
		cpu.clock_tick();
		assert_eq!(cpu.registers.PC, 0x0A0B);
		assert_eq!(cpu.registers.S, 0xFD);
		let pc_after_lsb = cpu.pop_stack();
		let pc_after_msb = cpu.pop_stack();
		let pc_after = ((pc_after_msb as u16) << 8) | (pc_after_lsb as u16);
		assert_eq!(pc_after, pc_before + 2); 
		assert_eq!(cpu.registers.S, 0xFF);
	}

	#[test]
	fn test_indexed_absolute() {
		let mut cpu = initialize(load_program_absolute_indexed_with_carry);

		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusRegisterBits::CARRY), true);
		cpu.clock_tick();
		cpu.clock_tick();
		

		cpu.clock_tick();
		assert_eq!(cpu.bus.memory.read(0x20AB), 0xFF);

		cpu.clock_tick();
	}

	#[test]
	fn test_all_transfers() {
		// I know, its stupid test. But more tests = better. It will all payout eventually.
		let mut cpu = initialize(load_program_transfers);

		assert_eq!(cpu.registers.S, 0xFF);
		cpu.clock_tick();
		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0xAA);
		assert_eq!(cpu.registers.X, 0xAA);
		assert_eq!(cpu.registers.Y, 0xAA);
		cpu.clock_tick();
		assert_ne!(cpu.registers.X, 0xAA);
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0x00);
		cpu.clock_tick();
		assert_ne!(cpu.registers.A, 0x00);
		cpu.clock_tick();
		assert_eq!(cpu.registers.X, 0xBB);
		cpu.clock_tick();
		assert_eq!(cpu.registers.S, 0xBB);
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0xAA);
		cpu.clock_tick();
	}

}