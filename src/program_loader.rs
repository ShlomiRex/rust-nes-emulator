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

pub fn load_program_absolute_store(rom: &mut [u8;65_536]) -> u8 {
	/*
	SEI
	CLD
	LDX #$AB
	STX $2000
	STX $2001
	NOP
	*/
	write_rom(rom, "78 d8 a2 ab 8e 00 20 8e 01 20 ea");
	6
}

pub fn load_program_index_increment(rom: &mut [u8;65_536]) -> u8 {
	/*
	LDX #$FE
	INX
	INX
	NOP
	*/
	write_rom(rom, "a2 fe e8 e8 ea");
	4
}

pub fn load_program_zeropage_store_load_and_memory_increment(rom: &mut [u8;65_536]) -> u8 {
	/*
	LDX #$FE
	STX $0A
	INC $0A
	INC $0A
	NOP
	*/
	write_rom(rom, "a2 fe 86 0a e6 0a e6 0a ea");
	5
}

pub fn load_program_zeropage_x(rom: &mut [u8;65_536]) -> u8 {
	/*
	LDX #$FE 	; Load index X
	STX $0A 	; Store index in zero page (non-indexed)
	LDA $0A 	; Load from zero page (non-indexed)

	LDX #$FF 	; We want to access zeropage indexed (0x05 + 0x05 = 0x0A)
	LDA #$00 	; Reset A so we can examine if we got to the same address
	LDA $0B,X 	; We load at zeropage, address: (0xFF + 0x0B = 0x0A, zeropage is overflowing 1 byte). So A should be 0xFE.

	LDX #$0B 	; We want to access 0x0A again, in next instruction we access: 0x0B + 0xFF = 0x0A memory.
	ADC $FF,X 	; We add A (0xFE) with memory (at 0x0A), which is (0xFE + 0xFE = 0x1FC, but carry is 1, so its 0xFC)

	NOP
	*/
	write_rom(rom, "a2 fe 86 0a a5 0a a2 ff a9 00 b5 0b a2 0b 75 ff ea");
	9
}
