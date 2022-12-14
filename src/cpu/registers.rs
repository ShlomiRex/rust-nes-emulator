use std::fmt;
use crate::common::bits;

/// # CPU Registers
/// (Chip: 6502), wikipedia: https://en.wikipedia.org/wiki/MOS_Technology_6502#Registers
#[derive(Default)]
#[allow(non_snake_case)]
pub struct Registers {
	pub A: u8,							// accumulator
	pub X: u8,							// index register
	pub Y: u8,							// index register
	pub P: ProcessorStatus,				// processor status flag bits
	pub S: u8,							// stack pointer
	pub PC: u16,						// program counter
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A: {:#X},\tX: {:#X},\tY: {:#X},\tS: {:#X},\tPC: {:#X},\tP: {}", self.A, self.X, self.Y, self.S, self.PC, self.P)
    }
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
#[derive(Debug)]
#[repr(u8)]
pub enum ProcessorStatusBits {
	CARRY,
	ZERO,
	InterruptDisable,
	DECIMAL,
	BREAK,
	UNUSED,		// By the datasheet it looks like its always 1.
	OVERFLOW,
	NEGATIVE
}

pub struct ProcessorStatus {
	pub flags: u8
}

impl Default for ProcessorStatus {
    fn default() -> Self {
		// Set 'UNUSED' flag to 1. Its the standard.
        Self { flags: 0b0010_0000 }
    }
}

impl ProcessorStatus {
	pub fn set(&mut self, bit: ProcessorStatusBits, value: bool) {
		bits::set(&mut self.flags, bit as u8, value);
	}

	pub fn get(&self, bit: ProcessorStatusBits) -> bool {
		bits::get(self.flags, bit as u8)
	}

	/// Sets the N bitflag, depending on arithmetic result. Its common for all the instructions.
	pub fn modify_n(&mut self, value: u8) {
		// If last bit (7) is 1, its negative
		self.set(ProcessorStatusBits::NEGATIVE, (value >> 7) == 1);
	}

	/// Sets the Z bitflag, depending on arithmetic result. Its common for all the instructions.
	pub fn modify_z(&mut self, value: u8) {
		// If value is 0, zero flag is 1
		self.set(ProcessorStatusBits::ZERO, value == 0); 
	}
}

impl fmt::Display for ProcessorStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NV-BDIZC {:08b}", self.flags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
	use ProcessorStatusBits::*;

    #[test]
    fn processor_status_register_test() {
		let mut registers = Registers::default();

		assert!(registers.P.get(CARRY) == false);
		registers.P.set(CARRY, true);
		assert!(registers.P.get(CARRY) == true);

		assert!(registers.P.get(NEGATIVE) == false);
		registers.P.set(NEGATIVE, true);
		assert!(registers.P.get(NEGATIVE) == true);
		registers.P.set(NEGATIVE, false);
		assert!(registers.P.get(NEGATIVE) == false);
		registers.P.set(NEGATIVE, false);
		assert!(registers.P.get(NEGATIVE) == false);
    }

	#[test]
	fn p_register_format_test() {
		// I had trouble with format. But someone helped me: https://www.reddit.com/r/learnrust/comments/ypyquy/format_u8_to_display_binary_without_0b_and_with/
		let mut p = ProcessorStatus { flags: 0 };

		p.flags = 0b1100_0110;
		assert_eq!(format!("{p}"), "NV-BDIZC 11000110");

		p.flags = 0b0000_0010;
		assert_eq!(format!("{p}"), "NV-BDIZC 00000010");
	}
}