use crate::memory::write_rom;

/// This will populate the ROM with TOLOWER subroutine. \
/// The TOLOWER routine is described here: https://en.wikipedia.org/wiki/MOS_Technology_6502#Registers \
/// "which copies a null-terminated character string from one location to another, converting upper-case letter characters to lower-case letters."
/// Returns the amount of assembly lines / code written in ROM.
pub fn load_program_tolower(rom_memory: &mut [u8;65_536]) -> u8 {
	rom_memory[0] 	= 0xA0; //LDY #$00
	rom_memory[1] 	= 0x00;
	rom_memory[2] 	= 0xB1;	//LDA (SRC),Y
	rom_memory[3] 	= 0x80;
	rom_memory[4] 	= 0xF0;	//BEQ DONE
	rom_memory[5] 	= 0x11;
	rom_memory[6] 	= 0xC9;	//CMP #'A'
	rom_memory[7] 	= 0x41;
	rom_memory[8] 	= 0x90;	//BCC SKIP
	rom_memory[9] 	= 0x06;
	rom_memory[10] 	= 0xC9;	//CMP #'Z'+1
	rom_memory[11] 	= 0x58;
	rom_memory[12] 	= 0xB0; //BCS SKIP
	rom_memory[13] 	= 0x02;
	rom_memory[14] 	= 0x09; //ORA #%00100000
	rom_memory[15] 	= 0x20;
	rom_memory[16] 	= 0x91; //STA (DST),Y
	rom_memory[17] 	= 0x82;
	rom_memory[18] 	= 0xC8; //INY
	rom_memory[19] 	= 0xD0; //BNE LOOP
	rom_memory[20] 	= 0xED;
	rom_memory[21] 	= 0x38; //SEC
	rom_memory[22] 	= 0x60; //RTS
	rom_memory[23] 	= 0x91; //STA (DST),Y
	rom_memory[24] 	= 0x82;
	rom_memory[25] 	= 0x18; //CLC
	rom_memory[26] 	= 0x60; //RTS

	16
}

/// Program is taken from the first example here: https://skilldrick.github.io/easy6502/#first-program
/// Returns the amount of assembly lines / code written in ROM.
pub fn load_program_helloworld(rom_memory: &mut [u8;65_536]) -> u8 {
	// LDA #$01
	// STA $0200
	// LDA #$05
	// STA $0201
	// LDA #$08
	// STA $0202

	write_rom(rom_memory, "a9 01 8d 00 02 a9 05 8d 01 02 a9 08 8d 02 02");
	6
}

pub fn load_program_stack_operations(rom: &mut [u8;65_536]) -> u8 {
	// Push "8c ab" onto stack and get it
	/*
		LDA #$8C
		PHA
		LDA #$AB
		PHA
		PLA
		PLA
		NOP
	*/
	write_rom(rom, "A9 8C 48 A9 AB 48 68 68 EA");
	7
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
	*/
	write_rom(rom, "D8 A9 09 18 69 02 F8 A9 09 18 69 02");
	8
}

pub fn load_program_lda_negative(rom: &mut [u8;65_536]) -> u8 {
	/*
	LDA #$FF
	NOP
	*/
	write_rom(rom, "A9 FF EA");
	2
}

pub fn load_program_adc_carry(rom: &mut [u8;65_536]) -> u8 {
	/*
	LDA #$FF
	ADC #$81
	NOP
	*/
	write_rom(rom, "a9 ff 69 81 ea");
	3
}

pub fn load_program_adc_decimal(rom: &mut [u8;65_536]) -> u8 {
	/*
	SED
	LDA #$09
	CLC
	ADC #$02
	NOP
	*/
	write_rom(rom, "f8 a9 09 18 69 02 ea");
	5
}

pub fn load_program_reset_sp(rom: &mut [u8;65_536]) -> u8 {
	/*
	PLA
	NOP
	*/
	write_rom(rom, "68 ea");
	2
}

pub fn load_program_overflow_1(rom: &mut [u8;65_536]) -> u8 {
	/*
	CLC
	LDA #$80
	ADC #$FF
	NOP
	*/
	write_rom(rom, "18 a9 80 69 ff ea");
	4
}

pub fn load_program_overflow_2(rom: &mut [u8;65_536]) -> u8 {
	/*
	CLC      ; 127 + 1 = 128, returns V = 1
	LDA #$7F
	ADC #$01
	NOP
	*/
	write_rom(rom, "18 a9 7f 69 01 ea");
	4
}

