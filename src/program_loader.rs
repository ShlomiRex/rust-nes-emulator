use crate::memory::write_rom;

// Each function loads a program to memory, and returns amount of assembly lines used.

/// Basic stack operations; Push A, pull A.
pub fn load_program_stack(rom: &mut [u8;65_536]) -> u8 {
	// Push "8c ab" onto stack and get it
	/*
		LDA #$8C
		PHA
		LDA #$AB
		PHA
		PLA
		PLA
		PLA ; This will overflow the stack pointer
		NOP
	*/
	write_rom(rom, "A9 8C 48 A9 AB 48 68 68 68 EA");
	8
}

/// Load A register; Changes P bitflags: negative, zero.
pub fn load_program_lda(rom: &mut [u8;65_536]) -> u8 {
	/*
	LDA #$FF
	LDA #$00
	NOP
	*/
	write_rom(rom, "A9 FF A9 00 EA");
	3
}

pub fn load_program_adc(rom: &mut [u8;65_536]) -> u8 {
	/*
	CLD
	LDA #$09
	CLC
	ADC #$02 	; A will be 0x0B, as expected (0x9 + 0x2 = 0xB)

	SED
	LDA #$09
	CLC
	ADC #$02 	; A will be 0x11, because decimal bitflag is enabled (i.e. represent the sum in 'decimal' (11) form, not hex (0xB) form)
				; NOTE: In NES, the decimal mode is not used. Perhaps I should remove this feature?
	
	CLD
	LDA #$FF
	ADC #$81	; Sets CARRY flag

	CLC
	LDA #$80
	ADC #$FF 	; Sets OVERRFLOW flag (and carry)
	
	CLV
	CLC      ; 127 + 1 = 128, returns V = 1
	LDA #$7F
	ADC #$01

	NOP
	*/
	write_rom(rom, "d8 a9 09 18 69 02 f8 a9 09 18 69 02 d8 a9 ff 69 81 18 a9 80 69 ff b8 18 a9 7f 69 01 ea");
	19
}
