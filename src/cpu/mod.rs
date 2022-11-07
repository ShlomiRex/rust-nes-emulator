mod registers;
mod decoder;

pub use registers::Registers;

use crate::bus::Bus;


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
	registers: Registers,
	bus: Box<Bus>
}

impl CPU {
	pub fn new(bus: Box<Bus>) -> Self {
		let registers: Registers = Registers::default();
		CPU {
			registers,
			bus
		}
	}

	/// A single clock cycle is executed here.
	fn clock_tick(&self) {
		//TODO: Complete

		// Read next instruction.
		let opcode = self.bus.rom.read(self.registers.PC); // Read at address of Program Counter (duh!)
		let instruction = decoder::decode_opcode(opcode);
		
	}

	fn nmi_interrupt(&self) {
		//TODO: Complete
	}

	fn irq_interrupt(&self) {
		//TODO: Complete
	}
}

#[cfg(test)]
mod tests {
    use super::*;
	use registers::ProcessorStatusRegisterBits::*;
	use crate::bus::Bus;

    #[test]
    fn processor_status_register_test() {
		let bus: Bus = Bus::new();
		let mut cpu = CPU::new(Box::new(bus));

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