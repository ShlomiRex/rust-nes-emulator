/// The decoder's purpose is to take OPCODE and translate it to the appropriate instruction.
// https://www.masswerk.at/6502/6502_instruction_set.html

/// All possible CPU instructions. This is written like in 6502 assembler.
#[derive(PartialEq)]
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

/// Taken from wikipedia.org
#[derive(PartialEq)]
pub enum AddressingMode {
	IMPLIED, 		// 1 byte
	ABSOLUTE, 		// 3 bytes
	INDEXED, 		// 3 bytes
	ZEROPAGE, 		// 2 bytes
	RELATIVE, 		// 2 bytes
	ACCUMULATOR, 	// 1 byte
	INDIRECTX, 		// 2 bytes
	INDIRECTY, 		// 2 bytes
	IMMEDIATE , 	// 2 bytes
}

/// Decode CPU instruction, probably from ROM or something. \
/// Returns the Instruction (like in assembly), Addressing Mode, Bytes, Cycles.
pub fn decode_opcode(opcode: u8) -> (Instructions, AddressingMode, u8, u8) {
	// No need to deconstruct the opcode into this, since we can match all 255 possible opcodes with hex anyway.
	// let CC = opcode & 0b11;				// define the opcode
	// let BBB = (opcode >> 2) & 0b111;	// defines addressing mode
	// let AAA = (opcode >> 5) & 0b111;	// define the opcode

	println!("Test");

	match opcode {
		0x00 => (Instructions::BRK, AddressingMode::IMPLIED, 1, 2),
		0x18 => (Instructions::CLC, AddressingMode::IMPLIED, 1, 2),

		_ => {
			//TODO: For now we panic, but we must handle this later. What happens when illegal instruction is called in real NES?
			panic!("Could not decode instruction, opcode: {}", opcode);
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