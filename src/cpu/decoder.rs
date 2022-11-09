/// The decoder's purpose is to take OPCODE and translate it to the appropriate instruction.
// https://www.masswerk.at/6502/6502_instruction_set.html

use log::error;
use std::{fmt, ops::Add};

/// All possible CPU instructions. This is written like in 6502 assembler.
#[derive(PartialEq, Debug)]
pub enum Instructions {
	ADC, // add with carry
	AND, // and (with accumulator)
	ASL, // arithmetic shift left
	BCC, // branch on carry clear
	BCS, // branch on carry set
	BEQ, // branch on equal (zero set)
	BIT, // bit test
	BMI, // branch on minus (negative set)
	BNE, // branch on not equal (zero clear)
	BPL, // branch on plus (negative clear)
	BRK, // break / interrupt
	BVC, // branch on overflow clear
	BVS, // branch on overflow set
	CLC, // clear carry
	CLD, // clear decimal
	CLI, // clear interrupt disable
	CLV, // clear overflow
	CMP, // compare (with accumulator)
	CPX, // compare with X
	CPY, // compare with Y
	DEC, // decrement
	DEX, // decrement X
	DEY, // decrement Y
	EOR, // exclusive or (with accumulator)
	INC, // increment
	INX, // increment X
	INY, // increment Y
	JMP, // jump
	JSR, // jump subroutine
	LDA, // load accumulator
	LDX, // load X
	LDY, // load Y
	LSR, // logical shift right
	NOP, // no operation
	ORA, // or with accumulator
	PHA, // push accumulator
	PHP, // push processor status (SR)
	PLA, // pull accumulator
	PLP, // pull processor status (SR)
	ROL, // rotate left
	ROR, // rotate right
	RTI, // return from interrupt
	RTS, // return from subroutine
	SBC, // subtract with carry
	SEC, // set carry
	SED, // set decimal
	SEI, // set interrupt disable
	STA, // store accumulator
	STX, // store X
	STY, // store Y
	TAX, // transfer accumulator to X
	TAY, // transfer accumulator to Y
	TSX, // transfer stack pointer to X
	TXA, // transfer X to accumulator
	TXS, // transfer X to stack pointer
	TYA  // transfer Y to accumulator
}

/// Taken from wikipedia.org \
/// After further reading here: https://en.wikipedia.org/wiki/MOS_Technology_6502#Registers
/// 
/// | Mode | Description |
/// |---|---|
/// | IMPLIED | No data is needed to be fetched to execute the instruction |
/// | ABSOLUTE | The next 2 bytes after opcode is the memory, which indicates memory location in absolute integer |
/// | INDEXED | Indexed addressing modes use the X or Y register to help determine the address. |
/// | ZEROPAGE | Zero page is only the first 256 bytes of memory (absolute address of $0-$FF). The next byte after opcode is the memory address to take the data from. For example, `LDA $35` will load the 2 bytes at the memory location of 35. Advantage of zero-page are two - the instruction takes one less byte to specify, and it executes in less CPU cycles.|
/// | RELATIVE | The next byte after opcode is offset. Add program counter with offset to get relative address. |
/// | ACCUMULATOR | The memory needed to execute instruction is inside A register |
/// | INDIRECT | The `JMP` instruction is the only instruction which uses indirect. The instruction is 3 bytes long. Consider: `JMP ($1000)`, and at memory $1000, $1001 the bytes are: `52 3a`, then the PC will be set to $3a52. |
/// | INDIRECTX |  |
/// | INDIRECTY |  |
/// | IMMEDIATE | Data defined in next byte after opcode |
#[derive(PartialEq, Debug)]
pub enum AddressingMode {
	IMPLIED, 		// 1 byte
	ABSOLUTE, 		// 3 bytes
	ABSOLUTEX,
	ABSOLUTEY,
	// INDEXED, 		// 3 bytes
	ZEROPAGE, 		// 2 bytes
	ZEROPAGEX,
	// ZEROPAGEY,
	RELATIVE, 		// 2 bytes
	ACCUMULATOR, 	// 1 byte
	INDIRECT, 
	INDIRECTX, 		// 2 bytes
	INDIRECTY, 		// 2 bytes
	IMMEDIATE , 	// 2 bytes
}


/// Instruction's cycles can be changed if some conditions are met. \
/// Explanation:\
///
/// 
/// | Enum               | Description                                                                                              |
/// |--------------------|----------------------------------------------------------------------------------------------------------|
/// | NONE               | don't change amount of cycles                                                                            |
/// | PageBoundryCrossed | add 1 to cycles if page boundary is crossed                                                              |
/// | BranchOccursOn     | add 2 to cycles if branch occurs on same page <br> or add 2 to cycles if branch occurs to different page |
/// 
/// 
pub enum CycleOops {
	NONE,
	PageBoundryCrossed,
	BranchOccursOn
}

impl fmt::Display for CycleOops {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match *self {
			CycleOops::NONE => write!(f, "No"),
			_=> write!(f, "Yes")
		}
    }
}

/// Decode CPU instruction, probably from ROM or something. \
/// Returns the Instruction (like in assembly), Addressing Mode, Bytes, Cycles.
pub fn decode_opcode(opcode: u8) -> (Instructions, AddressingMode, u8, u8, CycleOops) {
	// No need to deconstruct the opcode into this, since we can match all 255 possible opcodes with hex anyway.
	// let CC = opcode & 0b11;				// define the opcode
	// let BBB = (opcode >> 2) & 0b111;	// defines addressing mode
	// let AAA = (opcode >> 5) & 0b111;	// define the opcode

	match opcode {
		0x00 => (Instructions::BRK, AddressingMode::IMPLIED, 		1, 2, CycleOops::NONE),
		0x01 => (Instructions::ORA, AddressingMode::INDIRECTX, 		2, 6, CycleOops::NONE),
		0x05 => (Instructions::ORA, AddressingMode::ZEROPAGE, 		2, 3, CycleOops::NONE),
		0x06 => (Instructions::ASL, AddressingMode::ZEROPAGE, 		2, 5, CycleOops::NONE),
		0x08 => (Instructions::PHP, AddressingMode::IMPLIED, 		1, 3, CycleOops::NONE),
		0x09 => (Instructions::ORA, AddressingMode::IMMEDIATE, 		2, 2, CycleOops::NONE),
		0x0A => (Instructions::ASL, AddressingMode::ACCUMULATOR, 	1, 2, CycleOops::NONE),
		0x0D => (Instructions::ORA, AddressingMode::ABSOLUTE, 		3, 4, CycleOops::NONE),
		0x0E => (Instructions::ASL, AddressingMode::ABSOLUTE, 		3, 6, CycleOops::NONE),
		0x10 => (Instructions::BPL, AddressingMode::RELATIVE, 		2, 2, CycleOops::BranchOccursOn),
		0x11 => (Instructions::ORA, AddressingMode::INDIRECTY, 		2, 5, CycleOops::PageBoundryCrossed),
		0x15 => (Instructions::ORA, AddressingMode::ZEROPAGEX, 		2, 4, CycleOops::NONE),
		0x16 => (Instructions::ASL, AddressingMode::ZEROPAGEX, 		2, 6, CycleOops::NONE),
		0x18 => (Instructions::CLC, AddressingMode::IMPLIED, 		1, 2, CycleOops::NONE),
		0x19 => (Instructions::ORA, AddressingMode::ABSOLUTEY, 		3, 4, CycleOops::PageBoundryCrossed),
		0x1D => (Instructions::ORA, AddressingMode::ABSOLUTEX, 		3, 4, CycleOops::PageBoundryCrossed),
		0x1E => (Instructions::ASL, AddressingMode::ABSOLUTEX, 		3, 7, CycleOops::NONE),
		0x20 => (Instructions::JSR, AddressingMode::ABSOLUTE, 		3, 6, CycleOops::NONE),
		0x21 => (Instructions::AND, AddressingMode::INDIRECTX, 		2, 6, CycleOops::NONE),
		0x24 => (Instructions::BIT, AddressingMode::ZEROPAGE, 		2, 3, CycleOops::NONE),
		0x25 => (Instructions::AND, AddressingMode::ZEROPAGE, 		2, 3, CycleOops::NONE),
		0x26 => (Instructions::ROL, AddressingMode::ZEROPAGE, 		2, 5, CycleOops::NONE),
		0x28 => (Instructions::PLP, AddressingMode::IMPLIED, 		1, 4, CycleOops::NONE),
		0x29 => (Instructions::AND, AddressingMode::IMMEDIATE, 		2, 2, CycleOops::NONE),
		0x2A => (Instructions::ROL, AddressingMode::ACCUMULATOR, 	1, 2, CycleOops::NONE),
		0x2C => (Instructions::BIT, AddressingMode::ABSOLUTE, 		3, 4, CycleOops::NONE),
		0x2D => (Instructions::AND, AddressingMode::ABSOLUTE, 		3, 4, CycleOops::NONE),
		0x2E => (Instructions::ROL, AddressingMode::ABSOLUTE, 		3, 6, CycleOops::NONE),
		0x30 => (Instructions::BMI, AddressingMode::RELATIVE, 		2, 2, CycleOops::BranchOccursOn),
		0x31 => (Instructions::AND, AddressingMode::INDIRECTY, 		2, 5, CycleOops::PageBoundryCrossed),
		0x35 => (Instructions::AND, AddressingMode::ZEROPAGEX, 		2, 4, CycleOops::NONE),
		0x36 => (Instructions::ROL, AddressingMode::ZEROPAGEX, 		2, 6, CycleOops::NONE),
		0x38 => (Instructions::SEC, AddressingMode::IMPLIED, 		1, 2, CycleOops::NONE),
		0x39 => (Instructions::AND, AddressingMode::ABSOLUTEY, 		3, 4, CycleOops::PageBoundryCrossed),
		0x3D => (Instructions::AND, AddressingMode::ABSOLUTEX, 		3, 4, CycleOops::PageBoundryCrossed),
		0x3E => (Instructions::ROL, AddressingMode::ABSOLUTEX, 		3, 7, CycleOops::NONE),
		0x40 => (Instructions::RTI, AddressingMode::IMMEDIATE, 		1, 6, CycleOops::NONE),
		0x41 => (Instructions::EOR, AddressingMode::INDIRECTX, 		2, 6, CycleOops::NONE),
		0x45 => (Instructions::EOR, AddressingMode::ZEROPAGE, 		2, 3, CycleOops::NONE),
		0x46 => (Instructions::LSR, AddressingMode::ZEROPAGE, 		2, 5, CycleOops::NONE),
		0x48 => (Instructions::PHA, AddressingMode::IMPLIED, 		1, 3, CycleOops::NONE),
		0x49 => (Instructions::EOR, AddressingMode::IMMEDIATE, 		2, 2, CycleOops::NONE),
		0x4A => (Instructions::LSR, AddressingMode::ACCUMULATOR, 	1, 2, CycleOops::NONE),
		0x4C => (Instructions::JMP, AddressingMode::ABSOLUTE, 		3, 3, CycleOops::NONE),
		0x4D => (Instructions::EOR, AddressingMode::ABSOLUTE, 		3, 4, CycleOops::NONE),
		0x4E => (Instructions::LSR, AddressingMode::ABSOLUTE, 		3, 6, CycleOops::NONE),
		0x50 => (Instructions::BVC, AddressingMode::RELATIVE, 		2, 2, CycleOops::BranchOccursOn),
		0x51 => (Instructions::EOR, AddressingMode::INDIRECTY, 		2, 5, CycleOops::BranchOccursOn),
		0x55 => (Instructions::EOR, AddressingMode::ZEROPAGEX, 		2, 4, CycleOops::NONE),
		0x56 => (Instructions::LSR, AddressingMode::ZEROPAGEX, 		2, 6, CycleOops::NONE),
		0x58 => (Instructions::CLI, AddressingMode::IMPLIED, 		1, 2, CycleOops::NONE),
		0x59 => (Instructions::EOR, AddressingMode::ABSOLUTEY, 		3, 4, CycleOops::PageBoundryCrossed),
		0x5D => (Instructions::EOR, AddressingMode::ABSOLUTEX, 		3, 4, CycleOops::PageBoundryCrossed),
		0x5E => (Instructions::LSR, AddressingMode::ABSOLUTEX, 		3, 7, CycleOops::NONE),
		0x60 => (Instructions::RTS, AddressingMode::IMPLIED, 		1, 6, CycleOops::NONE),
		0x61 => (Instructions::ADC, AddressingMode::INDIRECTX, 		2, 6, CycleOops::NONE),
		0x65 => (Instructions::ADC, AddressingMode::ZEROPAGE, 		2, 3, CycleOops::NONE),
		0x66 => (Instructions::ROR, AddressingMode::ZEROPAGE, 		2, 5, CycleOops::NONE),
		0x68 => (Instructions::PLA, AddressingMode::IMPLIED, 		1, 4, CycleOops::NONE),
		0x69 => (Instructions::ADC, AddressingMode::IMMEDIATE, 		2, 2, CycleOops::NONE),
		0x6A => (Instructions::ROR, AddressingMode::ACCUMULATOR, 	1, 2, CycleOops::NONE),
		0x6C => (Instructions::JMP, AddressingMode::INDIRECT, 		3, 5, CycleOops::NONE),
		0x6D => (Instructions::ADC, AddressingMode::ABSOLUTE, 		3, 4, CycleOops::NONE),
		0x6E => (Instructions::ROR, AddressingMode::ABSOLUTE, 		3, 6, CycleOops::NONE),
		0x8D => (Instructions::STA, AddressingMode::ABSOLUTE, 		3, 4, CycleOops::NONE),
		0x90 => (Instructions::BCC, AddressingMode::RELATIVE, 		2, 2, CycleOops::BranchOccursOn),
		0x91 => (Instructions::STA, AddressingMode::INDIRECTY, 		2, 6, CycleOops::NONE),
		0xA0 => (Instructions::LDY, AddressingMode::IMMEDIATE, 		2, 2, CycleOops::NONE),
		0xA9 => (Instructions::LDA, AddressingMode::IMMEDIATE, 		2, 2, CycleOops::NONE),
		0xB0 => (Instructions::BCS, AddressingMode::RELATIVE, 		2, 2, CycleOops::BranchOccursOn),
		0xB1 => (Instructions::LDA, AddressingMode::INDIRECTY, 		2, 5, CycleOops::PageBoundryCrossed),
		0xC8 => (Instructions::INY, AddressingMode::IMPLIED, 		1, 2, CycleOops::NONE),
		0xC9 => (Instructions::CMP, AddressingMode::IMMEDIATE, 		2, 2, CycleOops::NONE),
		0xD0 => (Instructions::BNE, AddressingMode::RELATIVE, 		2, 2, CycleOops::BranchOccursOn),
		0xF0 => (Instructions::BEQ, AddressingMode::RELATIVE, 		2, 2, CycleOops::BranchOccursOn),
		_ => {
			//TODO: For now we panic, but we must handle this later. What happens when illegal instruction is called in real NES?
			error!("Could not decode instruction, opcode: {:#X}", opcode);
			panic!();
		}
	}
}	

#[cfg(test)]
mod tests {
    use super::decode_opcode;
	use super::Instructions;
	use super::AddressingMode;

    #[test]
	fn test_decoder() {
		let result = decode_opcode(0x18); 		// Clear Carry Flag
		assert!(result.0 == Instructions::CLC && result.1 == AddressingMode::IMPLIED && result.2 == 1 && result.3 == 2);
	}
}