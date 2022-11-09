/// The decoder's purpose is to take OPCODE and translate it to the appropriate instruction.
// https://www.masswerk.at/6502/6502_instruction_set.html

use log::error;
use std::fmt;
use ProcessorStatusRegisterBitChanges::*;

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
	ZEROPAGEY,
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
pub enum OopsCycle {
	NONE,
	PageBoundryCrossed,
	BranchOccursOn
}

impl fmt::Display for OopsCycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match *self {
			OopsCycle::NONE => write!(f, "No"),
			_=> write!(f, "Yes")
		}
    }
}

pub enum ProcessorStatusRegisterBitChanges {
	NotModified,
	MODIFIED,
	SET,
	CLEARED,
	M6,
	M7,
	FromStack
}

impl fmt::Display for ProcessorStatusRegisterBitChanges {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let letter = match self {
			NotModified => '-',
			MODIFIED => '+',
			SET => '1',
			CLEARED => '0',
			M6 => '6',
			M7 => '7',
			FromStack => 'S'
		};
        write!(f, "{}", letter)
    }
}

// The tuple represents the P flag, like so: N Z C I D V (the order matters)
// Each bit flag can be of type: ProcessorStatusRegisterBitChanges.
// pub struct PBitflagsChange(ProcessorStatusRegisterBitChanges, ProcessorStatusRegisterBitChanges, ProcessorStatusRegisterBitChanges, 
// 	ProcessorStatusRegisterBitChanges, ProcessorStatusRegisterBitChanges, ProcessorStatusRegisterBitChanges);
pub struct PBitflagsChange {
	pub n: ProcessorStatusRegisterBitChanges,
	pub z: ProcessorStatusRegisterBitChanges,
	pub c: ProcessorStatusRegisterBitChanges,
	pub i: ProcessorStatusRegisterBitChanges,
	pub d: ProcessorStatusRegisterBitChanges,
	pub v: ProcessorStatusRegisterBitChanges
}

impl fmt::Display for PBitflagsChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NZCIDV {}{}{}{}{}{}", self.n, self.z, self.c, self.i, self.d, self.v)
    }
}

/// Decode CPU instruction, probably from ROM or something. \
/// Returns the Instruction (like in assembly), Addressing Mode, Bytes, Cycles.
pub fn decode_opcode(opcode: u8) -> (Instructions, AddressingMode, u8, u8, OopsCycle, PBitflagsChange) {

	// Each variable is pre-fabricated object that will be used in the match statement next.
	// I do this in order to not go insane about filling 151 lines with 6 options. (151*6 = 906 options!!!). And I would go crazy when I add illegal opcodes.

	// N Z C I D V 			- - 1 - - -
	let __1___: PBitflagsChange = 		PBitflagsChange{ n: NotModified, 	z: NotModified, 	c: SET, 			i: NotModified, 	d: NotModified, 	v: NotModified 	};
	// N Z C I D V 			- - - 1 - -
	let ___1__: PBitflagsChange = 		PBitflagsChange{ n: NotModified, 	z: NotModified, 	c: NotModified, 	i: SET, 			d: NotModified, 	v: NotModified 	};
	// N Z C I D V 			- - - - 1 -
	let ____1_: PBitflagsChange = 		PBitflagsChange{ n: NotModified, 	z: NotModified, 	c: NotModified, 	i: NotModified, 	d: SET, 			v: NotModified 	};
	// N Z C I D V 			+ + - - - -
	let MM____: PBitflagsChange = 		PBitflagsChange{ n: MODIFIED, 		z: MODIFIED, 		c: NotModified, 	i: NotModified, 	d: NotModified, 	v: NotModified 	};
	// N Z C I D V 			+ + + - - -
	let MMM___: PBitflagsChange = 		PBitflagsChange{ n: MODIFIED, 		z: MODIFIED, 		c: MODIFIED, 		i: NotModified, 	d: NotModified, 	v: NotModified 	};
	// N Z C I D V 			+ + + - - +
	let MMM__M: PBitflagsChange = 		PBitflagsChange{ n: MODIFIED, 		z: MODIFIED, 		c: MODIFIED, 		i: NotModified, 	d: NotModified, 	v: MODIFIED 	};
	// N Z C I D V 			- - - - - -
	let ______: PBitflagsChange = 		PBitflagsChange{ n: NotModified, 	z: NotModified, 	c: NotModified, 	i: NotModified, 	d: NotModified, 	v: NotModified 	};
	// N Z C I D V 			M7 + - - - M6
	let m7M___m6: PBitflagsChange = 	PBitflagsChange{ n: M7, 			z: MODIFIED, 		c: NotModified, 	i: NotModified, 	d: NotModified, 	v: M6 			};
	// N Z C I D V 			  from stack
	let from_stack: PBitflagsChange = 	PBitflagsChange{ n: FromStack, 		z: FromStack, 		c: FromStack, 		i: FromStack, 		d: FromStack, 		v: FromStack 	};
	// N Z C I D V 			0 + + - - -
	let zMM___: PBitflagsChange = 		PBitflagsChange{ n: CLEARED, 		z: MODIFIED, 		c: MODIFIED, 		i: NotModified, 	d: NotModified, 	v: NotModified	};
	// N Z C I D V 			- - 0 - - -
	let __0___: PBitflagsChange = 		PBitflagsChange{ n: NotModified, 	z: NotModified, 	c: CLEARED, 		i: NotModified, 	d: NotModified, 	v: NotModified 	};
	// N Z C I D V 			- - - 0 - -
	let ___0__: PBitflagsChange = 		PBitflagsChange{ n: NotModified, 	z: NotModified, 	c: NotModified, 	i: CLEARED, 		d: NotModified, 	v: NotModified 	};
	// N Z C I D V 			- - - - 0 -
	let ____0_: PBitflagsChange = 		PBitflagsChange{ n: NotModified, 	z: NotModified, 	c: NotModified, 	i: NotModified, 	d: CLEARED, 		v: NotModified 	};
	// N Z C I D V 			- - - - - 0
	let _____0: PBitflagsChange = 		PBitflagsChange{ n: NotModified, 	z: NotModified, 	c: NotModified, 	i: NotModified, 	d: NotModified, 	v: CLEARED 		};


	match opcode {
		0x00 => (Instructions::BRK, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					___1__),
		0x01 => (Instructions::ORA, AddressingMode::INDIRECTX, 		2, 6, OopsCycle::NONE, 					MM____),
		0x05 => (Instructions::ORA, AddressingMode::ZEROPAGE, 		2, 3, OopsCycle::NONE, 					MM____),
		0x06 => (Instructions::ASL, AddressingMode::ZEROPAGE, 		2, 5, OopsCycle::NONE, 					MMM___),
		0x08 => (Instructions::PHP, AddressingMode::IMPLIED, 		1, 3, OopsCycle::NONE, 					______),
		0x09 => (Instructions::ORA, AddressingMode::IMMEDIATE, 		2, 2, OopsCycle::NONE, 					MM____),
		0x0A => (Instructions::ASL, AddressingMode::ACCUMULATOR, 	1, 2, OopsCycle::NONE, 					MMM___),
		0x0D => (Instructions::ORA, AddressingMode::ABSOLUTE, 		3, 4, OopsCycle::NONE, 					MM____),
		0x0E => (Instructions::ASL, AddressingMode::ABSOLUTE, 		3, 6, OopsCycle::NONE, 					MMM___),
		0x10 => (Instructions::BPL, AddressingMode::RELATIVE, 		2, 2, OopsCycle::BranchOccursOn, 		______),
		0x11 => (Instructions::ORA, AddressingMode::INDIRECTY, 		2, 5, OopsCycle::PageBoundryCrossed, 	MM____),
		0x15 => (Instructions::ORA, AddressingMode::ZEROPAGEX, 		2, 4, OopsCycle::NONE, 					MM____),
		0x16 => (Instructions::ASL, AddressingMode::ZEROPAGEX, 		2, 6, OopsCycle::NONE, 					MMM___),
		0x18 => (Instructions::CLC, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					__0___),
		0x19 => (Instructions::ORA, AddressingMode::ABSOLUTEY, 		3, 4, OopsCycle::PageBoundryCrossed, 	MM____),
		0x1D => (Instructions::ORA, AddressingMode::ABSOLUTEX, 		3, 4, OopsCycle::PageBoundryCrossed, 	MM____),
		0x1E => (Instructions::ASL, AddressingMode::ABSOLUTEX, 		3, 7, OopsCycle::NONE, 					MMM___),
		0x20 => (Instructions::JSR, AddressingMode::ABSOLUTE, 		3, 6, OopsCycle::NONE, 					______),
		0x21 => (Instructions::AND, AddressingMode::INDIRECTX, 		2, 6, OopsCycle::NONE, 					MM____),
		0x24 => (Instructions::BIT, AddressingMode::ZEROPAGE, 		2, 3, OopsCycle::NONE, 					m7M___m6),
		0x25 => (Instructions::AND, AddressingMode::ZEROPAGE, 		2, 3, OopsCycle::NONE, 					MM____),
		0x26 => (Instructions::ROL, AddressingMode::ZEROPAGE, 		2, 5, OopsCycle::NONE, 					MMM___),
		0x28 => (Instructions::PLP, AddressingMode::IMPLIED, 		1, 4, OopsCycle::NONE, 					from_stack),
		0x29 => (Instructions::AND, AddressingMode::IMMEDIATE, 		2, 2, OopsCycle::NONE, 					MM____),
		0x2A => (Instructions::ROL, AddressingMode::ACCUMULATOR, 	1, 2, OopsCycle::NONE, 					MMM___),
		0x2C => (Instructions::BIT, AddressingMode::ABSOLUTE, 		3, 4, OopsCycle::NONE, 					m7M___m6),
		0x2D => (Instructions::AND, AddressingMode::ABSOLUTE, 		3, 4, OopsCycle::NONE, 					MM____),
		0x2E => (Instructions::ROL, AddressingMode::ABSOLUTE, 		3, 6, OopsCycle::NONE, 					MMM___),
		0x30 => (Instructions::BMI, AddressingMode::RELATIVE, 		2, 2, OopsCycle::BranchOccursOn, 		______),
		0x31 => (Instructions::AND, AddressingMode::INDIRECTY, 		2, 5, OopsCycle::PageBoundryCrossed, 	MM____),
		0x35 => (Instructions::AND, AddressingMode::ZEROPAGEX, 		2, 4, OopsCycle::NONE, 					MM____),
		0x36 => (Instructions::ROL, AddressingMode::ZEROPAGEX, 		2, 6, OopsCycle::NONE, 					MMM___),
		0x38 => (Instructions::SEC, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					__1___),
		0x39 => (Instructions::AND, AddressingMode::ABSOLUTEY, 		3, 4, OopsCycle::PageBoundryCrossed, 	MM____),
		0x3D => (Instructions::AND, AddressingMode::ABSOLUTEX, 		3, 4, OopsCycle::PageBoundryCrossed, 	MM____),
		0x3E => (Instructions::ROL, AddressingMode::ABSOLUTEX, 		3, 7, OopsCycle::NONE, 					MMM___),
		0x40 => (Instructions::RTI, AddressingMode::IMMEDIATE, 		1, 6, OopsCycle::NONE, 					from_stack),
		0x41 => (Instructions::EOR, AddressingMode::INDIRECTX, 		2, 6, OopsCycle::NONE, 					MM____),
		0x45 => (Instructions::EOR, AddressingMode::ZEROPAGE, 		2, 3, OopsCycle::NONE, 					MM____),
		0x46 => (Instructions::LSR, AddressingMode::ZEROPAGE, 		2, 5, OopsCycle::NONE, 					zMM___),
		0x48 => (Instructions::PHA, AddressingMode::IMPLIED, 		1, 3, OopsCycle::NONE, 					______),
		0x49 => (Instructions::EOR, AddressingMode::IMMEDIATE, 		2, 2, OopsCycle::NONE, 					MM____),
		0x4A => (Instructions::LSR, AddressingMode::ACCUMULATOR, 	1, 2, OopsCycle::NONE, 					zMM___),
		0x4C => (Instructions::JMP, AddressingMode::ABSOLUTE, 		3, 3, OopsCycle::NONE, 					______),
		0x4D => (Instructions::EOR, AddressingMode::ABSOLUTE, 		3, 4, OopsCycle::NONE, 					MM____),
		0x4E => (Instructions::LSR, AddressingMode::ABSOLUTE, 		3, 6, OopsCycle::NONE, 					zMM___),
		0x50 => (Instructions::BVC, AddressingMode::RELATIVE, 		2, 2, OopsCycle::BranchOccursOn, 		______),
		0x51 => (Instructions::EOR, AddressingMode::INDIRECTY, 		2, 5, OopsCycle::BranchOccursOn, 		MM____),
		0x55 => (Instructions::EOR, AddressingMode::ZEROPAGEX, 		2, 4, OopsCycle::NONE, 					MM____),
		0x56 => (Instructions::LSR, AddressingMode::ZEROPAGEX, 		2, 6, OopsCycle::NONE, 					zMM___),
		0x58 => (Instructions::CLI, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					___0__),
		0x59 => (Instructions::EOR, AddressingMode::ABSOLUTEY, 		3, 4, OopsCycle::PageBoundryCrossed, 	MM____),
		0x5D => (Instructions::EOR, AddressingMode::ABSOLUTEX, 		3, 4, OopsCycle::PageBoundryCrossed, 	MM____),
		0x5E => (Instructions::LSR, AddressingMode::ABSOLUTEX, 		3, 7, OopsCycle::NONE, 					zMM___),
		0x60 => (Instructions::RTS, AddressingMode::IMPLIED, 		1, 6, OopsCycle::NONE, 					______),
		0x61 => (Instructions::ADC, AddressingMode::INDIRECTX, 		2, 6, OopsCycle::NONE, 					MMM__M),
		0x65 => (Instructions::ADC, AddressingMode::ZEROPAGE, 		2, 3, OopsCycle::NONE, 					MMM__M),
		0x66 => (Instructions::ROR, AddressingMode::ZEROPAGE, 		2, 5, OopsCycle::NONE, 					MMM___),
		0x68 => (Instructions::PLA, AddressingMode::IMPLIED, 		1, 4, OopsCycle::NONE, 					MM____),
		0x69 => (Instructions::ADC, AddressingMode::IMMEDIATE, 		2, 2, OopsCycle::NONE, 					MMM__M),
		0x6A => (Instructions::ROR, AddressingMode::ACCUMULATOR, 	1, 2, OopsCycle::NONE, 					MMM___),
		0x6C => (Instructions::JMP, AddressingMode::INDIRECT, 		3, 5, OopsCycle::NONE, 					______),
		0x6D => (Instructions::ADC, AddressingMode::ABSOLUTE, 		3, 4, OopsCycle::NONE, 					MMM__M),
		0x6E => (Instructions::ROR, AddressingMode::ABSOLUTE, 		3, 6, OopsCycle::NONE, 					MMM___),
		0x70 => (Instructions::BVS, AddressingMode::RELATIVE, 		2, 2, OopsCycle::BranchOccursOn, 		______),
		0x71 => (Instructions::ADC, AddressingMode::INDIRECTY, 		2, 5, OopsCycle::PageBoundryCrossed, 	MMM__M),
		0x75 => (Instructions::ADC, AddressingMode::ZEROPAGEX, 		2, 4, OopsCycle::NONE, 					MMM__M),
		0x76 => (Instructions::ROR, AddressingMode::ZEROPAGEX, 		2, 6, OopsCycle::NONE, 					MMM___),
		0x78 => (Instructions::SEI, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					___1__),
		0x79 => (Instructions::ADC, AddressingMode::ABSOLUTEY, 		3, 4, OopsCycle::PageBoundryCrossed, 	MMM__M),
		0x7D => (Instructions::ADC, AddressingMode::ABSOLUTEX, 		3, 4, OopsCycle::PageBoundryCrossed, 	MMM__M),
		0x7E => (Instructions::ROR, AddressingMode::ABSOLUTEX, 		3, 7, OopsCycle::NONE, 					MMM___),
		0x81 => (Instructions::STA, AddressingMode::INDIRECTX, 		2, 6, OopsCycle::NONE, 					______),
		0x84 => (Instructions::STY, AddressingMode::ZEROPAGE, 		2, 3, OopsCycle::NONE, 					______),
		0x85 => (Instructions::STA, AddressingMode::ZEROPAGE, 		2, 3, OopsCycle::NONE, 					______),
		0x86 => (Instructions::STX, AddressingMode::ZEROPAGE, 		2, 3, OopsCycle::NONE, 					______),
		0x88 => (Instructions::DEY, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					MM____),
		0x8A => (Instructions::TXA, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					MM____),
		0x8C => (Instructions::STY, AddressingMode::ABSOLUTE, 		3, 4, OopsCycle::NONE, 					______),
		0x8D => (Instructions::STA, AddressingMode::ABSOLUTE, 		3, 4, OopsCycle::NONE, 					______),
		0x8E => (Instructions::STX, AddressingMode::ABSOLUTE, 		3, 4, OopsCycle::NONE, 					______),
		0x90 => (Instructions::BCC, AddressingMode::RELATIVE, 		2, 2, OopsCycle::BranchOccursOn, 		______),
		0x91 => (Instructions::STA, AddressingMode::INDIRECTY, 		2, 6, OopsCycle::NONE, 					______),
		0x94 => (Instructions::STY, AddressingMode::ZEROPAGEX, 		2, 4, OopsCycle::NONE, 					______),
		0x95 => (Instructions::STA, AddressingMode::ZEROPAGEX, 		2, 4, OopsCycle::NONE, 					______),
		0x96 => (Instructions::STX, AddressingMode::ZEROPAGEY, 		2, 4, OopsCycle::NONE, 					______),
		0x98 => (Instructions::TYA, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					MM____),
		0x99 => (Instructions::STA, AddressingMode::ABSOLUTEY, 		3, 5, OopsCycle::NONE, 					______),
		0x9A => (Instructions::TXS, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					______),
		0x9D => (Instructions::STA, AddressingMode::ABSOLUTEX, 		3, 5, OopsCycle::NONE, 					______),
		0xA0 => (Instructions::LDY, AddressingMode::IMMEDIATE, 		2, 2, OopsCycle::NONE, 					MM____),
		0xA1 => (Instructions::LDA, AddressingMode::INDIRECTX, 		2, 6, OopsCycle::NONE, 					MM____),
		0xA2 => (Instructions::LDX, AddressingMode::IMMEDIATE, 		2, 2, OopsCycle::NONE, 					MM____),
		0xA4 => (Instructions::LDY, AddressingMode::ZEROPAGE, 		2, 3, OopsCycle::NONE, 					MM____),
		0xA5 => (Instructions::LDA, AddressingMode::ZEROPAGE, 		2, 3, OopsCycle::NONE, 					MM____),
		0xA6 => (Instructions::LDX, AddressingMode::ZEROPAGE, 		2, 3, OopsCycle::NONE, 					MM____),
		0xA8 => (Instructions::TAY, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					MM____),
		0xA9 => (Instructions::LDA, AddressingMode::IMMEDIATE, 		2, 2, OopsCycle::NONE, 					MM____),
		0xAA => (Instructions::TAX, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					MM____),
		0xAC => (Instructions::LDY, AddressingMode::ABSOLUTE, 		3, 4, OopsCycle::NONE, 					MM____),
		0xAD => (Instructions::LDA, AddressingMode::ABSOLUTE, 		3, 4, OopsCycle::NONE, 					MM____),
		0xAE => (Instructions::LDX, AddressingMode::ABSOLUTE, 		3, 4, OopsCycle::NONE, 					MM____),
		0xB0 => (Instructions::BCS, AddressingMode::RELATIVE, 		2, 2, OopsCycle::BranchOccursOn, 		______),
		0xB1 => (Instructions::LDA, AddressingMode::INDIRECTY, 		2, 5, OopsCycle::PageBoundryCrossed, 	MM____),
		0xB4 => (Instructions::LDY, AddressingMode::ZEROPAGEX, 		2, 4, OopsCycle::NONE, 					MM____),
		0xB5 => (Instructions::LDA, AddressingMode::ZEROPAGEX, 		2, 4, OopsCycle::NONE, 					MM____),
		0xB6 => (Instructions::LDX, AddressingMode::ZEROPAGEY, 		2, 4, OopsCycle::NONE, 					MM____),
		0xB8 => (Instructions::CLV, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					_____0),
		0xB9 => (Instructions::LDA, AddressingMode::ABSOLUTEY, 		3, 4, OopsCycle::PageBoundryCrossed, 	MM____),
		0xBA => (Instructions::TSX, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					MM____),
		0xBC => (Instructions::LDY, AddressingMode::ABSOLUTEX, 		3, 4, OopsCycle::PageBoundryCrossed, 	MM____),
		0xBD => (Instructions::LDA, AddressingMode::ABSOLUTEX, 		3, 4, OopsCycle::PageBoundryCrossed, 	MM____),
		0xBE => (Instructions::LDX, AddressingMode::ABSOLUTEY, 		3, 4, OopsCycle::PageBoundryCrossed, 	MM____),
		0xC0 => (Instructions::CPY, AddressingMode::IMMEDIATE, 		2, 2, OopsCycle::NONE, 					MMM___),
		0xC1 => (Instructions::CMP, AddressingMode::INDIRECTX, 		2, 6, OopsCycle::NONE, 					MMM___),
		0xC4 => (Instructions::CPY, AddressingMode::ZEROPAGE, 		2, 3, OopsCycle::NONE, 					MMM___),
		0xC5 => (Instructions::CMP, AddressingMode::ZEROPAGE, 		2, 3, OopsCycle::NONE, 					MMM___),
		0xC6 => (Instructions::DEC, AddressingMode::ZEROPAGE, 		2, 5, OopsCycle::NONE, 					MM____),
		0xC8 => (Instructions::INY, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					MM____),
		0xC9 => (Instructions::CMP, AddressingMode::IMMEDIATE, 		2, 2, OopsCycle::NONE, 					MMM___),
		0xCA => (Instructions::DEX, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					MM____),
		0xCC => (Instructions::CPY, AddressingMode::ABSOLUTE, 		3, 4, OopsCycle::NONE, 					MMM___),
		0xCD => (Instructions::CMP, AddressingMode::ABSOLUTE, 		3, 4, OopsCycle::NONE, 					MMM___),
		0xCE => (Instructions::DEC, AddressingMode::ABSOLUTE, 		3, 6, OopsCycle::NONE, 					MM____),
		0xD0 => (Instructions::BNE, AddressingMode::RELATIVE, 		2, 2, OopsCycle::BranchOccursOn, 		______),
		0xD1 => (Instructions::CMP, AddressingMode::INDIRECTY, 		2, 5, OopsCycle::PageBoundryCrossed, 	MMM___),
		0xD5 => (Instructions::CMP, AddressingMode::ZEROPAGEX, 		2, 4, OopsCycle::NONE, 					MMM___),
		0xD6 => (Instructions::DEC, AddressingMode::ZEROPAGEX, 		2, 6, OopsCycle::NONE, 					MM____),
		0xD8 => (Instructions::CLD, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					____0_),
		0xD9 => (Instructions::CMP, AddressingMode::ABSOLUTEY, 		3, 4, OopsCycle::PageBoundryCrossed, 	MMM___),
		0xDD => (Instructions::CMP, AddressingMode::ABSOLUTEX, 		3, 4, OopsCycle::PageBoundryCrossed, 	MMM___),
		0xDE => (Instructions::DEC, AddressingMode::ABSOLUTEX, 		3, 7, OopsCycle::NONE, 					MM____),
		0xE0 => (Instructions::CPX, AddressingMode::IMMEDIATE, 		2, 2, OopsCycle::NONE, 					MMM___),
		0xE1 => (Instructions::SBC, AddressingMode::INDIRECTX, 		2, 6, OopsCycle::NONE, 					MMM__M),
		0xE4 => (Instructions::CPX, AddressingMode::ZEROPAGE, 		2, 3, OopsCycle::NONE, 					MMM___),
		0xE5 => (Instructions::SBC, AddressingMode::ZEROPAGE, 		2, 3, OopsCycle::NONE, 					MMM__M),
		0xE6 => (Instructions::INC, AddressingMode::ZEROPAGE, 		2, 5, OopsCycle::NONE, 					MM____),
		0xE8 => (Instructions::INX, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					MM____),
		0xE9 => (Instructions::SBC, AddressingMode::IMMEDIATE, 		2, 2, OopsCycle::NONE, 					MMM__M),
		0xEA => (Instructions::NOP, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					______),
		0xEC => (Instructions::CPX, AddressingMode::ABSOLUTE, 		3, 4, OopsCycle::NONE, 					MMM___),
		0xED => (Instructions::SBC, AddressingMode::ABSOLUTE, 		3, 4, OopsCycle::NONE, 					MMM__M),
		0xEE => (Instructions::INC, AddressingMode::ABSOLUTE, 		3, 6, OopsCycle::NONE, 					MM____),
		0xF0 => (Instructions::BEQ, AddressingMode::RELATIVE, 		2, 2, OopsCycle::BranchOccursOn, 		______),
		0xF1 => (Instructions::SBC, AddressingMode::INDIRECTY, 		2, 5, OopsCycle::PageBoundryCrossed, 	MMM__M),
		0xF5 => (Instructions::SBC, AddressingMode::ZEROPAGEX, 		2, 4, OopsCycle::NONE, 					MMM__M),
		0xF6 => (Instructions::INC, AddressingMode::ZEROPAGEX, 		2, 6, OopsCycle::NONE, 					MM____),
		0xF8 => (Instructions::SED, AddressingMode::IMPLIED, 		1, 2, OopsCycle::NONE, 					____1_),
		0xF9 => (Instructions::SBC, AddressingMode::ABSOLUTEY, 		3, 4, OopsCycle::PageBoundryCrossed, 	MMM__M),
		0xFD => (Instructions::SBC, AddressingMode::ABSOLUTEX, 		3, 4, OopsCycle::PageBoundryCrossed, 	MMM__M),
		0xFE => (Instructions::INC, AddressingMode::ABSOLUTEX, 		3, 7, OopsCycle::NONE, 					MM____),
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