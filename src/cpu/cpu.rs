use core::panic;
use log::{debug, error, warn};


use crate::apu::apu::APU;
use crate::cartridge::Cartridge;
use crate::cpu::registers::{Registers, ProcessorStatusBits, ProcessorStatus};
use crate::cpu::decoder::{OopsCycle, Instructions, AddressingMode, decode_opcode};
use crate::mmu::MMU;
use crate::ppu::ppu::PPU;

use hex::FromHex;

pub struct LowerMemory {
	pub zero_page: [u8; 0xFF],
	pub stack: [u8; 0xFF],
	pub ram: [u8; 0x5FF]
}

pub struct CPU {
	registers: Registers,
	cycles: u64,
	mmu: MMU,
	cartridge: Cartridge,
	ppu: PPU,
	lower_memory: LowerMemory,
	apu: APU
}

impl CPU {
	pub fn new(mmu: MMU, cartridge: Cartridge, ppu: PPU, apu: APU) -> Self {
		let registers: Registers = Registers::default();
		let lower_memory = LowerMemory { 
			zero_page: [0;0xFF], 
			stack: [0;0xFF], 
			ram: [0;0x5FF] 
		};
		let mut cpu = CPU {
			registers,
			cycles: 0,
			mmu,
			cartridge,
			ppu,
			lower_memory,
			apu
		};
		cpu.res_interrupt();
		cpu
	}

	/// A single clock cycle is executed here.
	/// Original NES CPU needs multiple cycles to execute instruction.
	/// Emulation does not do that; Its much simpler to do everything at once, and emulate the cycles.
	pub fn clock_tick(&mut self) {
		debug!("Tick, cycle: {}", self.cycles);
		debug!("{}", self.registers);

		// Read next instruction.
		let opcode = self.read_memory(self.registers.PC); // Read at address of Program Counter (duh!)
		let instruction = decode_opcode(opcode);

		let instr = instruction.0;
		let addrmode = instruction.1;
		let bytes = instruction.2;
		let cycles = instruction.3;
		let oops_cycle = instruction.4;

		debug!("{:#X}: {:?}\t{:?}\tBytes: {}, Cycles: {}, Oops cycle: {}", opcode, instr, addrmode, bytes, cycles, oops_cycle);

		self.execute_instruction(&instr, addrmode);

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

	/// The main brains of the CPU. Execute instruction.
	fn execute_instruction(&mut self, instr: &Instructions, addrmode: AddressingMode) {
		//The main brains of the CPU. Execute instruction.
		match instr {
			Instructions::LDX | 
			Instructions::LDY | 
			Instructions::LDA => {
				/*
				LDX:
				Load Index X with Memory
				M -> X

				LDY:
				Load Index Y with Memory
				M -> Y

				LDA:
				Load Accumulator with Memory
				M -> A
				*/
				let fetched_memory = self.fetch_memory(&addrmode);
				if fetched_memory != 0 {
					println!("ok");
				}
				if *instr == Instructions::LDX {
					self.registers.X = fetched_memory;
				} else if *instr == Instructions::LDY {
					self.registers.Y = fetched_memory;
				} else {
					// LDA
					self.registers.A = fetched_memory;
				}

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
				self.registers.P.set(ProcessorStatusBits::CARRY, true);
			}
			Instructions::CLC => {
				// Clear Carry Flag
				self.registers.P.set(ProcessorStatusBits::CARRY, false);
			}
			Instructions::SED => {
				// Set Decimal Flag
				self.registers.P.set(ProcessorStatusBits::DECIMAL, true);
			}
			Instructions::CLD => {
				// Clear Decimal Mode
				self.registers.P.set(ProcessorStatusBits::DECIMAL, false);
			}
			Instructions::SEI => {
				// Set Interrupt Disable Status
				self.registers.P.set(ProcessorStatusBits::InterruptDisable, true);
			}
			Instructions::CLI => {
				// Clear Interrupt Disable Bit
				self.registers.P.set(ProcessorStatusBits::InterruptDisable, false);
			}
			Instructions::CLV => {
				// Clear Overflow Flag
				self.registers.P.set(ProcessorStatusBits::OVERFLOW, false);
			}
			Instructions::ADC => {
				// Add Memory to Accumulator with Carry
				// A + M + C -> A, C
				// NOTE: This is the first instruction that actually does 'complex' arithmetic
				// After reading a lot of forums, its actually the most complex thing to emulate, I must understand this

				let fetched_memory = self.fetch_memory(&addrmode);

				let a = self.registers.A;
				let m = fetched_memory;
				let carry: u8 = self.registers.P.get(ProcessorStatusBits::CARRY) as u8;

				// Carry flag: Only for unsigned. If result is > 255, carry is set.
				// Overflow flag: Only if (Positive+Positive=Negative) or (Negative+Negative=Positive)

				// Perform regular unsigned addition, allowing arithmetic overflow.
				let first_addition = a.overflowing_add(m);
				let second_addition = first_addition.0.overflowing_add(carry);
				let mut result = second_addition.0;

				// Set A register.

				// Check decimal mode, check if CPU is in binary/decimal coded mode
				// TODO: I read that NES doesn't use this mode. Maybe remove it so I don't have any problems?
				if self.registers.P.get(ProcessorStatusBits::DECIMAL) {
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
				self.registers.P.set(ProcessorStatusBits::CARRY, new_carry);
				self.registers.P.set(ProcessorStatusBits::OVERFLOW, new_overflow);
			}
			Instructions::STX | 
			Instructions::STY | 
			Instructions::STA => {
				/*
				STX:
				Store Index X in Memory
				X -> M

				STY:
				Store Index Y in Memory
				Y -> M

				STA:
				Store Accumulator in Memory
				A -> M
				*/
				let addr = self.fetch_instruction_address(addrmode);
				if *instr == Instructions::STX {
					self.write_memory(addr, self.registers.X);
				} else if *instr == Instructions::STY {
					self.write_memory(addr, self.registers.Y);
				} else {
					//STA
					self.write_memory(addr, self.registers.A);
				}
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
			Instructions::INC | 
			Instructions::DEC => {
				/*
				INC:
				Increment Memory by One
				M + 1 -> M
				*/
				/*
				DEC:
				Decrement Memory by One
				M - 1 -> M
				*/
				let fetched_memory = self.fetch_memory(&addrmode);
				let new_memory = if *instr == Instructions::INC {
					fetched_memory.wrapping_add(1)
				} else {
					// DEC
					fetched_memory.wrapping_sub(1)
				};

				let addr = self.fetch_instruction_address(addrmode);
				self.write_memory(addr, new_memory);

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
				self.push_pc(2);

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
			Instructions::TAX => {
				// Transfer Accumulator to Index X
				// A -> X

				self.registers.X = self.registers.A;
				self.registers.P.modify_n(self.registers.X);
				self.registers.P.modify_z(self.registers.X);
			}
			Instructions::TAY => {
				// Transfer Accumulator to Index Y
				// A -> Y

				self.registers.Y = self.registers.A;
				self.registers.P.modify_n(self.registers.Y);
				self.registers.P.modify_z(self.registers.Y);
			}
			Instructions::TSX => {
				// Transfer Stack Pointer to Index X
				// SP -> X

				self.registers.X = self.registers.S;
				self.registers.P.modify_n(self.registers.X);
				self.registers.P.modify_z(self.registers.X);
			}
			Instructions::TXA => {
				// Transfer Index X to Accumulator
				// X -> A

				self.registers.A = self.registers.X;
				self.registers.P.modify_n(self.registers.A);
				self.registers.P.modify_z(self.registers.A);
			}
			Instructions::TXS => {
				// Transfer Index X to Stack Register
				// X -> SP

				self.registers.S = self.registers.X;
				// We don't modify N or Z.
			}
			Instructions::TYA => {
				// Transfer Index Y to Accumulator
				// Y -> A

				self.registers.A = self.registers.Y;
				self.registers.P.modify_n(self.registers.A);
				self.registers.P.modify_z(self.registers.A);
			}
			Instructions::AND => {
				// AND Memory with Accumulator
				// A AND M -> A

				let fetched_memory = self.fetch_memory(&addrmode);
				self.registers.A = self.registers.A & fetched_memory;
				self.registers.P.modify_n(self.registers.A);
				self.registers.P.modify_z(self.registers.A);
			}
			Instructions::ASL | Instructions::LSR => {
				/*
				ASL:
				Shift Left One Bit (Memory or Accumulator)
				C <- [76543210] <- 0
				*/
				/*
				LSR:
				Shift One Bit Right (Memory or Accumulator)
				0 -> [76543210] -> C
				*/

				// Memory can be register.
				let fetched_memory = self.fetch_memory(&addrmode);

				let result = if *instr == Instructions::ASL {
					fetched_memory << 1
				} else {
					// LSR
					fetched_memory >> 1
				};

				// Determine if shift overflowed (if yes, then set carry)
				// If last bit is 1, and we left shift, then that bit is the carry.
				let new_carry = (fetched_memory >> 7) == 1;

				// Now we need to know where to put the result. Register or memory?
				if addrmode == AddressingMode::ACCUMULATOR {
					self.registers.A = result;
				} else {
					// Get memory location.
					let addr = self.fetch_instruction_address(addrmode);
					self.write_memory(addr, result);
				}

				self.registers.P.modify_n(result);
				self.registers.P.modify_z(result);
				self.registers.P.set(ProcessorStatusBits::CARRY, new_carry);
			}
			Instructions::BIT => {
				// Test Bits in Memory with Accumulator

				//bits 7 and 6 of operand are transfered to bit 7 and 6 of SR (N,V);
				//the zero-flag is set to the result of operand AND accumulator.
				//A AND M, M7 -> N, M6 -> V

				let fetched_memory = self.fetch_memory(&addrmode);
				let result = self.registers.A & fetched_memory;
				let bit7 = (fetched_memory >> 7) == 1;
				let bit6 = ((fetched_memory >> 6) & 1) == 1;
				
				self.registers.P.set(ProcessorStatusBits::NEGATIVE, bit7);
				self.registers.P.set(ProcessorStatusBits::OVERFLOW, bit6);
				self.registers.P.modify_z(result);
			}
			Instructions::BMI | 
			Instructions::BPL | 
			Instructions::BNE | 
			Instructions::BVC | 
			Instructions::BVS |
			Instructions::BEQ |
			Instructions::BCS |
			Instructions::BCC
			 => {
				/*
				BMI:
				Branch on Result Minus
				branch on N = 1

				BPL:
				Branch on Result Plus
				branch on N = 0

				BNE:
				Branch on Result not Zero
				branch on Z = 0

				BVC:
				Branch on Overflow Clear
				branch on V = 0

				BVS:
				Branch on Overflow Set
				branch on V = 1

				BEQ:
 				Branch on Result Zero
				branch on Z = 1

				BCS:
				Branch on Carry Set
				branch on C = 1

				BCC:
				Branch on Carry Clear
				branch on C = 0
				*/

				if 
					(*instr == Instructions::BMI && self.registers.P.get(ProcessorStatusBits::NEGATIVE		) == true	) || 
					(*instr == Instructions::BPL && self.registers.P.get(ProcessorStatusBits::NEGATIVE		) == false	) ||
					(*instr == Instructions::BNE && self.registers.P.get(ProcessorStatusBits::ZERO			) == false	) ||
					(*instr == Instructions::BVC && self.registers.P.get(ProcessorStatusBits::OVERFLOW		) == false	) ||
					(*instr == Instructions::BVS && self.registers.P.get(ProcessorStatusBits::OVERFLOW		) == true	) ||
					(*instr == Instructions::BEQ && self.registers.P.get(ProcessorStatusBits::ZERO			) == true	) ||
					(*instr == Instructions::BCS && self.registers.P.get(ProcessorStatusBits::CARRY		) == true	) ||
					(*instr == Instructions::BCC && self.registers.P.get(ProcessorStatusBits::CARRY		) == false	)
				{
					let new_pc = self.read_instruction_relative_address();
					self.registers.PC = new_pc;
				}
			}
			Instructions::BRK => {
				// Force Break
				/*
				BRK initiates a software interrupt similar to a hardware
				interrupt (IRQ). The return address pushed to the stack is
				PC+2, providing an extra byte of spacing for a break mark
				(identifying a reason for the break.)
				The status register will be pushed to the stack with the break
				flag set to 1. However, when retrieved during RTI or by a PLP
				instruction, the break flag will be ignored.
				The interrupt disable flag is not set automatically.
				*/
				// interrupt,
				// push PC+2, push SR

				todo!();
				//self.push_pc(offset);
			}
			Instructions::DEX => {
				// Decrement Index X by One
				// X - 1 -> X

				self.registers.X = self.registers.X.wrapping_sub(1);
				self.registers.P.modify_n(self.registers.X);
				self.registers.P.modify_z(self.registers.X);
			}
			Instructions::DEY => {
				// Decrement Index Y by One
				// Y - 1 -> Y

				self.registers.Y = self.registers.Y.wrapping_sub(1);
				self.registers.P.modify_n(self.registers.Y);
				self.registers.P.modify_z(self.registers.Y);
			}
			Instructions::EOR | 
			Instructions::ORA => {
				/*
				EOR:
				Exclusive-OR Memory with Accumulator
				A EOR M -> A

				ORA:
				OR Memory with Accumulator
				A OR M -> A
				*/

				let fetched_memory = self.fetch_memory(&addrmode);
				let new_a = if *instr == Instructions::EOR {
					self.registers.A ^ fetched_memory
				} else {
					// ORA
					self.registers.A | fetched_memory
				};
				self.registers.A = new_a;
				self.registers.P.modify_n(self.registers.A);
				self.registers.P.modify_z(self.registers.A);
			}
			Instructions::PHP => {
				// Push Processor Status on Stack
				// The status register will be pushed with the break flag and bit 5 set to 1.
				// push SR

				self.push_p();
			}
			Instructions::PLP => {
				// Pull Processor Status from Stack
				// The status register will be pulled with the break flag and bit 5 ignored.
				// pull SR

				let p_flags = self.pop_stack();
				self.registers.P.flags = p_flags;
			}
			Instructions::RTI => {
				// Return from Interrupt
				// The status register is pulled with the break flag and bit 5 ignored. Then PC is pulled from the stack.
				// pull SR, pull PC

				let p = self.pop_stack();
				self.registers.P = ProcessorStatus {flags: p };
				let b = self.registers.P.get(ProcessorStatusBits::BREAK);
				self.registers.P.set(ProcessorStatusBits::BREAK, !b);
				
				self.registers.PC =  self.pop_pc();
			}
			Instructions::ROL => {
				// Rotate One Bit Left (Memory or Accumulator)
				// C <- [76543210] <- C
			}
			_ => {
				panic!("Could not execute instruction: {:?}, not implimented, yet", instr);
			}
		}
	}

	/// Reset interrupt. Address: $0xFFFC, $0xFFFD
	fn res_interrupt(&mut self) {
		debug!("Reset interrupt called");

		self.registers.A = 0;
		self.registers.X = 0;
		self.registers.Y = 0;
		self.registers.S = 0xFF;
		self.registers.P = ProcessorStatus::default();
		
		let new_addr = self.read_address_from_memory(0xFFFC);
		debug!("Jumping to interrupt address: {:#X}", new_addr);
		self.registers.PC = new_addr;

		self.cycles = 8;
	}

	/// Non-maskable interrupt. Address: $0xFFFA, $0xFFFB
	fn nmi_interrupt(&mut self) {
		debug!("NMI interrupt called");
		
		self.push_pc(0);

		self.registers.P.set(ProcessorStatusBits::BREAK, false);
		self.registers.P.set(ProcessorStatusBits::InterruptDisable, true);
		self.push_p();

		let new_addr = self.read_address_from_memory(0xFFFA);
		debug!("Jumping to interrupt address: {:#X}", new_addr);
		self.registers.PC = new_addr;

		self.cycles = 8;
	}

	/// Maskable interrupt. Address: $0xFFFE, $0xFFFF
	fn irq_interrupt(&mut self) {
		debug!("IRQ interrupt called");

		if self.registers.P.get(ProcessorStatusBits::InterruptDisable) == false {
			debug!("Executing IRQ interrupt");
			self.push_pc(0);

			//TODO: Not sure if we set break flag to 0. Research
			self.registers.P.set(ProcessorStatusBits::BREAK, false);
			self.registers.P.set(ProcessorStatusBits::InterruptDisable, true);
			self.push_p();

			let new_addr = self.read_address_from_memory(0xFFFE);
			debug!("Jumping to interrupt address: {:#X}", new_addr);
			self.registers.PC = new_addr;

			self.cycles = 7;
		}
	}

	fn push_stack(&mut self, data: u8) {
		self.write_memory(0x100 + self.registers.S as u16, data);
		self.registers.S -= 1;
		debug!("Pushed to stack: \t{:#X}", data);
	}

	fn pop_stack(&mut self) -> u8 {
		if self.registers.S == 0xFF {
			warn!("Stack pop: stack pointer is at beginning, overflowing stack pointer");
		}
		let head_addr: u16 = 0x100 + (self.registers.S as u16) + 1;  // we add 1 before the current SP points to get the head (the stack is down going)
		let res = self.read_memory(head_addr);
		self.registers.S = self.registers.S.wrapping_add(1);  // NOTE: We allow the programmer to overflow SP.
		debug!("Poped stack: \t{:#X}", res);
		res
	}

	/// Convert data from hex (example: 0x0B) to another hex (0x11), but is represented in 'decimal hex' form.
	fn decimal_mode(&self, data: u8) -> u8 {
		let hex_str = data.to_string();
		let decoded = <[u8; 1]>::from_hex(hex_str).expect("Could not convert decimal");
		decoded[0]
	}

	fn fetch_absolute_indexed(&mut self, index: u8) -> u8 {
		let addr = self.read_instruction_absolute_indexed_address(index);
		self.read_memory(addr)
	}

	fn fetch_zero_page_indexed(&mut self, index: u8) -> u8 {
		let instr_addr = self.read_instruction_zero_page_address();
		let addr = instr_addr.wrapping_add(index);
		self.read_memory(addr as u16)
	}

	/// Fetch memory required by the instruction. This can be in ROM (immediate, for example) or in RAM (absolute, for example), or CPU register.
	/// All load instructions use this.
	fn fetch_memory(&mut self, addrmode: &AddressingMode) -> u8 {
		match addrmode {
			AddressingMode::IMPLIED => {
				panic!("Instruction with implied addressing mode should never ask to fetch memory.");
			}
			AddressingMode::IMMEDIATE => {
				let addr = self.registers.PC + 1;
				let res = self.read_memory(addr);
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
				let res = self.read_memory(addr as u16);
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
	fn fetch_instruction_address(&mut self, addrmode: AddressingMode) -> u16 {
		match addrmode {
			AddressingMode::IMMEDIATE => {
				let res = self.read_memory(self.registers.PC + 1) as u16;
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
	fn read_instruction_absolute_address(&mut self) -> u16 {
		self.read_address_from_memory(self.registers.PC + 1)
	}

	/// Adds absolute address with index.
	fn read_instruction_absolute_indexed_address(&mut self, index: u8) -> u16 {
		self.read_instruction_absolute_address() + (index as u16)
	}

	/// Reads zero-page address stored in ROM at the current PC.
	fn read_instruction_zero_page_address(&mut self) -> u8 {
		self.read_memory(self.registers.PC + 1)
	}

	/// Returns address stored in memory, from the absolute address in ROM, at the current PC.
	fn read_instruction_indirect_address(&mut self) -> u16 {
		let indirect_addr = self.read_instruction_absolute_address();
		self.read_address_from_memory(indirect_addr)
	}

	/// Execute cmp instruction.
	/// Possible instructions: CMP (A register), CPX (X register), CPY (Y register).
	/// CMP is quite complex, which is why it has its own CPU function.
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

		self.registers.P.set(ProcessorStatusBits::NEGATIVE, new_n);
		self.registers.P.set(ProcessorStatusBits::ZERO, new_z);
		self.registers.P.set(ProcessorStatusBits::CARRY, new_c);
	}

	/// Read 2 bytes from memory that represent an address
	fn read_address_from_memory(&mut self, addr: u16) -> u16 {
		let lsb = self.read_memory(addr) as u16;
		let msb = self.read_memory(addr + 1) as u16;
		(msb << 8) | lsb
	}

	/// Calculate PC after applying relative offset. The offset is represented as signed integer.
	fn read_instruction_relative_address(&mut self) -> u16 {
		let offset = self.read_memory(self.registers.PC + 1);
		debug!("Relative offset: {:}", (offset as i8) as i16);
		self.registers.PC.wrapping_add_signed((offset as i8) as i16)
	}

	/// Push PC onto stack, adding offset to PC.
	fn push_pc(&mut self, offset: u16) {
		let pc_msb = (self.registers.PC.wrapping_add(offset) >> 8) as u8;
		let pc_lsb = (self.registers.PC.wrapping_add(offset)) as u8;
		self.push_stack(pc_msb); // store high
		self.push_stack(pc_lsb); // store low
	}

	/// Push processor status register onto stack
	fn push_p(&mut self) {
		self.push_stack(self.registers.P.flags);
	}

	/// Pops PC from stack.
	fn pop_pc(&mut self) -> u16 {
		let lsb = self.pop_stack() as u16;
		let msb = self.pop_stack() as u16;
		(msb << 8) | lsb
	}

	/// Generic function to read memory from CPU address space.
	fn read_memory(&mut self, addr: u16) -> u8 {
		self.mmu.read_request(&self.cartridge, &mut self.ppu, addr, &self.lower_memory)
	}

	/// Generic function to write memory from CPU address space.
	fn write_memory(&mut self, addr: u16, value: u8) {
		self.mmu.write_request(&mut self.ppu, addr, value, &mut self.lower_memory, &mut self.apu);
	}

}


#[cfg(test)]
mod tests {
    //use simple_logger::SimpleLogger;

    use crate::{
		program_loader::*, 
		cpu::registers::ProcessorStatusBits,
		nes::NES
	};

	fn initialize<'a>(f: fn(&mut [u8;1024*32]) -> u8) -> NES {
		let mut rom_memory: [u8; 1024*32] = [0;1024*32];
		f(&mut rom_memory);  // call f - load program

		let mut nes = NES::new_custom_prg_rom(rom_memory);
		nes.cpu.registers.PC = 0x8000; //TODO: Is it OK here?
		nes
	}

	// fn initialize_from_nes_rom(test_name: &str) -> CPU {
	// 	let mut path: String = String::from("6502asm_programs/tests/");
	// 	path += test_name;
	// 	path += ".nes";

	// 	let mut rom_parser = RomParser::new();
	// 	rom_parser.parse(path.as_str());
	// 	let prg_rom = rom_parser.prg_rom;
		
	// 	let lower_memory: [u8; 1024*32] = [0;1024*32];
	// 	let cartridge = Cartridge::new();
	// 	//let mm_ppu_registers = &mut lower_memory[0x2000..0x2008];
	// 	let ppu: PPU = PPU::new(&cartridge); // borrow cartridge
	// 	let mmu = MMU::new(lower_memory, &cartridge, &ppu);
	// 	let cpu = CPU::new(mmu);
	// 	cpu
	// }

	// NOTE: For each program, the last cpu tick is NOP, except for branch instructions, the last instruction in those is the stored instruction in memory.

	#[test]
	fn test_stack() {
		let mut nes = initialize(load_program_stack);
		let mut cpu = nes.cpu;
		
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0x8C);
		cpu.clock_tick();
		assert_eq!(cpu.read_memory(0x1FF), 0x8C);
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0xAB);
		cpu.clock_tick();
		assert_eq!(cpu.read_memory(0x1FE), 0xAB);
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
		let mut nes = initialize(load_program_lda);
		let mut cpu = nes.cpu;
		
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0xFF);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), true);
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), true);
		cpu.clock_tick();
	}

	#[test]
	fn test_adc() {
		let mut nes = initialize(load_program_adc);
		let mut cpu = nes.cpu;

		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::DECIMAL), false);
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0x09);
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), false);
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0x0B);
		
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::DECIMAL), true);
		cpu.clock_tick();
		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0x11);

		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::DECIMAL), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), false);
		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), true);
		assert_eq!(cpu.registers.A, 0x80);

		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), false);
		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::OVERFLOW), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), true);
		assert_eq!(cpu.registers.A, 0x7F);


		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::OVERFLOW), false);
		cpu.clock_tick();
		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::OVERFLOW), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), false);
		assert_eq!(cpu.registers.A, 0x80);

		cpu.clock_tick();
	}

	#[test]
	fn test_absolute_store() {
		let mut nes = initialize(load_program_absolute_store);
		let mut cpu = nes.cpu;

		cpu.clock_tick();
		cpu.clock_tick();
		cpu.clock_tick();

		assert_eq!(cpu.read_memory(0x2000), 0);
		cpu.clock_tick();
		assert_eq!(cpu.read_memory(0x2000), 0xAB);

		assert_eq!(cpu.read_memory(0x2001), 0);
		cpu.clock_tick();
		assert_eq!(cpu.read_memory(0x2001), 0xAB);
	}

	#[test]
	fn test_index_increment() {
		let mut nes = initialize(load_program_index_increment);
		let mut cpu = nes.cpu;

		cpu.clock_tick();
		assert_eq!(cpu.registers.X, 0xFE);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), true);
		cpu.clock_tick();
		assert_eq!(cpu.registers.X, 0xFF);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), true);
		cpu.clock_tick();
		assert_eq!(cpu.registers.X, 0x00);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), true);

		cpu.clock_tick();
	}

	#[test]
	fn test_zeropage_store_load_and_memory_increment() {
		let mut nes = initialize(load_program_zeropage_store_load_and_memory_increment);
		let mut cpu = nes.cpu;

		cpu.clock_tick();
		assert_eq!(cpu.registers.X, 0xFE);

		cpu.clock_tick();
		assert_eq!(cpu.read_memory(0x0A), 0xFE);

		cpu.clock_tick();
		assert_eq!(cpu.read_memory(0x0A), 0xFF);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), false);
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), true);
		assert_eq!(cpu.read_memory(0x0A), 0x00);

		cpu.clock_tick();
	}

	#[test]
	fn test_zeropage_x() {
		let mut nes = initialize(load_program_zeropage_x);
		let mut cpu = nes.cpu;

		cpu.clock_tick();
		cpu.clock_tick();
		cpu.clock_tick();

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.read_memory(0x0A), 0xFE);
		assert_ne!(cpu.registers.A, 0xFE);
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0xFE);

		cpu.clock_tick();
		assert_eq!(cpu.registers.X, 0x0B);
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0xFC);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), true);

		cpu.clock_tick();
	}

	#[test]
	fn test_absolute_indexed() {
		let mut nes = initialize(load_program_absolute_indexed);
		let mut cpu = nes.cpu;

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.read_memory(0x2000), 0x0A);
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
		let mut nes = initialize(load_program_jmp_absolute);
		let mut cpu = nes.cpu;

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.read_memory(0x0001), 0xF8); 	// Instruction SED (0xF8) is stored in memory location 0x0001. It's 1 byte long instruction.

		assert_ne!(cpu.registers.PC, 0x0001);
		cpu.clock_tick();
		assert_eq!(cpu.registers.PC, 0x0001);  // PC is at 0x0001

		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::DECIMAL), false);
		// Execute instruction stored in 0x0001
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::DECIMAL), true);
	}

	#[test]
	fn test_jmp_indirect() {
		let mut nes = initialize(load_program_jmp_indirect);
		let mut cpu = nes.cpu;

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.read_memory(0x00AB), 0x05);

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.read_memory(0x00AC), 0xFF);

		cpu.clock_tick();
		assert_eq!(cpu.registers.PC, 0xFF05);
	}

	#[test]
	fn test_cmp() {
		let mut nes = initialize(load_program_cmp);
		let mut cpu = nes.cpu;

		cpu.clock_tick();
		
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), false);
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), true);

		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), false);
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), true);

		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), false);

		cpu.clock_tick(); // LDA 0xAA: N=1, Z=C=0
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), true);

		cpu.clock_tick(); // LDA 0x00
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), false);

		cpu.clock_tick();
	}

	#[test]
	fn test_cpx() {
		// cpy is same...
		let mut nes = initialize(load_program_cpx);
		let mut cpu = nes.cpu;

		cpu.clock_tick();
		cpu.clock_tick();

		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), false);
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), true);

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), true);

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), false);

		cpu.clock_tick();
	}

	#[test]
	fn test_jsr() {
		let mut nes = initialize(load_program_jsr);
		let mut cpu = nes.cpu;
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
		let mut nes = initialize(load_program_absolute_indexed_with_carry);
		let mut cpu = nes.cpu;

		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), true);
		cpu.clock_tick();
		cpu.clock_tick();
		

		cpu.clock_tick();
		assert_eq!(cpu.read_memory(0x20AB), 0xFF);

		cpu.clock_tick();
	}

	#[test]
	fn test_all_transfers() {
		// I know, its stupid test. But more tests = better. It will all payout eventually.
		let mut nes = initialize(load_program_transfers);
		let mut cpu = nes.cpu;

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
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), false);
		cpu.clock_tick();
		assert_ne!(cpu.registers.A, 0x00);
		cpu.clock_tick();
		assert_eq!(cpu.registers.X, 0xBB);
		cpu.clock_tick();
		assert_eq!(cpu.registers.S, 0xBB);
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0xAA);

		// Run the program without debug and see whats the final flags. Easier than do it after the immediate instruction.
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), true);

		cpu.clock_tick();
	}

	#[test]
	fn test_and() {
		let mut nes = initialize(load_program_and);
		let mut cpu = nes.cpu;

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0xFF);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), true);

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0x83);

		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0x00);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), true);

		cpu.clock_tick();
	}

	#[test]
	fn test_asl() {
		let mut nes = initialize(load_program_asl);
		let mut cpu = nes.cpu;

		cpu.clock_tick();
		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0x04);

		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), false);
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), true);
		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0xF8);

		cpu.clock_tick(); // clc
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0x7F);
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0xFE);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), true);
		cpu.clock_tick();
		assert_eq!(cpu.registers.A, 0xFC);

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.read_memory(0x2000), 0x80);
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), false);
		assert_eq!(cpu.read_memory(0x2000), 0x00);

		cpu.clock_tick();
	}

	#[test]
	fn test_bcc() {
		let mut nes = initialize(load_program_bcc);
		let mut cpu = nes.cpu;

		cpu.clock_tick(); // CLC
		cpu.clock_tick(); // NOP
		let mut pc_before_bcc = cpu.registers.PC;
		cpu.clock_tick(); // BCC test
		let mut pc_after_bcc = cpu.registers.PC;
		assert!(pc_after_bcc - pc_before_bcc == 3);

		cpu.clock_tick(); // SEC
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::CARRY), true);
		pc_before_bcc = cpu.registers.PC;
		println!("{}", cpu.registers.PC);
		cpu.clock_tick(); // BCC success
		pc_after_bcc = cpu.registers.PC;
		println!("{}", cpu.registers.PC);
		assert!(pc_after_bcc - pc_before_bcc == 2);

		cpu.clock_tick(); // NOP (of success)
	}

	#[test]
	fn test_bit() {
		let nes = initialize(load_program_bit);
		let mut cpu = nes.cpu;

		cpu.clock_tick();
		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::OVERFLOW), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), true);

		cpu.clock_tick();
		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::OVERFLOW), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), true);

		cpu.clock_tick();
		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::OVERFLOW), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), false);

		cpu.clock_tick();
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::NEGATIVE), true);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::OVERFLOW), false);
		assert_eq!(cpu.registers.P.get(ProcessorStatusBits::ZERO), false);

		cpu.clock_tick();
	}

	// #[test]
	// fn test_bpl() {
	// 	let mut nes = initialize(load_program_bit);
	// 	let mut cpu = nes.cpu;
	// 	todo!();
	// }

	

	// fn test_page_crossed() {
	// 	let mut nes = initialize();

	// 	cpu.clock_tick();
	// }

}

