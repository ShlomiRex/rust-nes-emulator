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
#[derive(Default, Debug)]
#[allow(non_snake_case)]
pub struct Registers {
	pub A: u8, 			//accumulator
	pub X: u8, 			//index register
	pub Y: u8, 			//index register
	pub P: ProcessorStatusRegister, 			//processor status flag bits
	pub S: u8, 			//stack pointer
	pub PC: u16, 		//program counter
}

pub enum ProcessorStatusRegisterBits {
	CARRY,
	ZERO,
	DISABLE,
	DECIMAL,
	BREAK,
	UNUSED,		// By the datasheet it looks like its always 1.
	OVERFLOW,
	NEGATIVE
}

// TODO: Its possible this will slow down the CPU, because its called almsot each cycle.
impl ProcessorStatusRegisterBits {
	fn value(&self) -> usize {
		match *self {
			ProcessorStatusRegisterBits::CARRY 		=> 0,
			ProcessorStatusRegisterBits::ZERO 		=> 1,
			ProcessorStatusRegisterBits::DISABLE 	=> 2,
			ProcessorStatusRegisterBits::DECIMAL 	=> 3,
			ProcessorStatusRegisterBits::BREAK 		=> 4,
			ProcessorStatusRegisterBits::UNUSED 	=> 5,
			ProcessorStatusRegisterBits::OVERFLOW 	=> 6,
			ProcessorStatusRegisterBits::NEGATIVE 	=> 7
		}
	}
}

#[derive(Default, Debug)]
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