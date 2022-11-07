mod registers;
mod decoder;

pub use registers::Registers;


/// # 8-bit databus
/// Not to be confused with 'the bus', the data bus is 8 bits (8 input signals).
struct DataBus;
/// This signal allows to halt or single cycle the microprocessor on all cycles except write cycles.
struct ReadyInputSignal;
/// Request that an interrupt sequence begin within the microprocessor.
struct InterruptRequestSignal;
/// Request that a non-maskable interrupt sequence be generated within the microprocessor.
struct NonMaskableInterruptSignal;
/// Sets the overflow bit in the Processor Status Register.
struct SetOverflowSignal;

pub struct CPU {
	registers: Registers
}

impl CPU {
	pub fn new() -> Self {
		let registers: Registers = Registers::default();
		CPU {
			registers
		}
	}

	fn decode_opcode(&self, opcode: u8) {
		decoder::decode_opcode(opcode);
	}
}

#[cfg(test)]
mod tests {
    use super::*;
	use registers::ProcessorStatusRegisterBits::*;

    #[test]
    fn processor_status_register_test() {
		let mut cpu = CPU::new();

		assert!(cpu.registers.P.get(CARRY) == false);
		cpu.registers.P.set(CARRY, true);
		assert!(cpu.registers.P.get(CARRY) == true);

		assert!(cpu.registers.P.get(NEGATIVE) == false);
		cpu.registers.P.set(NEGATIVE, true);
		assert!(cpu.registers.P.get(NEGATIVE) == true);
		cpu.registers.P.set(NEGATIVE, false);
		assert!(cpu.registers.P.get(NEGATIVE) == false);
		cpu.registers.P.set(NEGATIVE, false);
		assert!(cpu.registers.P.get(NEGATIVE) == false);
    }
}