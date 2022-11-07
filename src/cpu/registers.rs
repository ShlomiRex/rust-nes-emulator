/// # CPU Registers
/// (Chip: 6502), wikipedia: https://en.wikipedia.org/wiki/MOS_Technology_6502#Registers
#[derive(Default)]
#[allow(non_snake_case)]
pub struct Registers {
	pub A: u8, 							//accumulator
	pub X: u8, 							//index register
	pub Y: u8, 							//index register
	pub P: ProcessorStatusRegister, 	//processor status flag bits
	pub S: u8, 							//stack pointer
	pub PC: u16, 						//program counter
}

/// # Processor Status Register
/// The P register contains 7 bit flags, and 1 bit unused (MSB)
/// 
/// | Bit | Symbol | Description |
/// |---|---|---|
/// | 7 | N | Negative |
/// | 6 | V | Overflow |
/// | 5 | - | Not used |
/// | 4 | B | Break |
/// | 3 | D | Decimal |
/// | 2 | I | Interrupt disable |
/// | 1 | Z | Zero |
/// | 0 | C | Carry |
#[allow(non_camel_case_types)]
pub enum ProcessorStatusRegisterBits {
	CARRY,
	ZERO,
	INTERRUPT_DISABLE,
	DECIMAL,
	BREAK,
	UNUSED,		// By the datasheet it looks like its always 1.
	OVERFLOW,
	NEGATIVE
}

use ProcessorStatusRegisterBits::*;
// TODO: Its possible this will slow down the CPU, because its called almsot each cycle.
impl ProcessorStatusRegisterBits {
	fn value(&self) -> usize {
		match *self {
			CARRY 				=> 0,
			ZERO 				=> 1,
			INTERRUPT_DISABLE 	=> 2,
			DECIMAL 			=> 3,
			BREAK 				=> 4,
			UNUSED 				=> 5,
			OVERFLOW 			=> 6,
			NEGATIVE 			=> 7
		}
	}
}

#[derive(Default)]
pub struct ProcessorStatusRegister {
	flags: u8
}

impl ProcessorStatusRegister {
	pub fn set(&mut self, bit: ProcessorStatusRegisterBits, value: bool) {
		let index = bit.value();
		if value {
			self.flags |= 1 << index;
		} else {
			self.flags &= !(1 << index);
		}
	}

	pub fn get(&self, bit: ProcessorStatusRegisterBits) -> bool {
		let index = bit.value();
		self.flags & (1 << index) != 0
	}
}