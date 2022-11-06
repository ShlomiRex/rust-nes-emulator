/// # CPU Registers
/// (Chip: 6502), wikipedia: https://en.wikipedia.org/wiki/MOS_Technology_6502#Registers
/// ## (P) Processor status flag bits
/// The P register contains 7 bit flags, and 1 bit unused (MSB)
/// 
/// | Bit | Symbol | Description |
/// |---|---|---|
/// | 7 | - | Not used |
/// | 6 | N | Negative |
/// | 5 | V | Overflow |
/// | 4 | B | Break |
/// | 3 | D | Decimal |
/// | 2 | I | Interrupt disable |
/// | 1 | Z | Zero |
/// | 0 | C | Carry |
#[derive(Debug)]
#[warn(non_snake_case)]
pub struct Registers {
	A: u8, //accumulator
	X: u8, //index register
	Y: u8, //index register
	P: u8, //processor status flag bits
	S: u8, //stack pointer
	PC: u16, //program counter
}

/// The CPU Address, 16 bits
pub type Address = u16;