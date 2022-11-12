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

pub fn load_program_absolute_indexed(rom: &mut [u8;65_536]) -> u8 {
	/*
	LDA #$0A 		; A=0x0A
	STA $ABCD		; $0xABCD = 0x0A
	LDX #$0D		; X=0x0D
	LDY $ABC0,X 	; Y = $(0xABC0 + 0x0D = 0xABCD) = 0x0A

	LDA #$00 		; A=0x00
	LDY #$FF 		; Y=0xFF
	LDA $AACE,Y 	; A = $(0xAACE + 0xFF = 0xABCD) = 0x0A

	NOP
	*/
	write_rom(rom, "a9 0a 8d cd ab a2 0d bc c0 ab a9 00 a0 ff b9 ce aa ea");
	8
}

pub fn load_program_jmp_absolute(rom: &mut [u8;65_536]) -> u8 {
	// Execute 1 byte long instruction at memory location 0x0001

	/*
	LDX #$F8 	; We load instruction 0xF8 (SED) to X
	STX $0001 	; Store instruction in $0001

	JMP 0001 	; Jump to $0001
	- 			; Execute the instruction in $0001 , DECIMAL bitflag is set. Note: We don't add assembly instruction here, because its out of reach. The PC changed.
	*/
	write_rom(rom, "a2 f8 8e 01 00 4c 01 00");
	4  	// 4 instructions, the last instruction should be executed (0xF8 = SED).
}

pub fn load_program_jmp_indirect(rom: &mut [u8;65_536]) -> u8 {
	/*
	LDA #$05
	STA $00AB

	LDA #$FF
	STA $00AC

	JMP ($00AB)
	*/
	write_rom(rom, "a9 05 8d ab 00 a9 ff 8d ac 00 6c ab 00");
	5
}

pub fn load_program_cmp(rom: &mut [u8;65_536]) -> u8 {
	/*
	LDA #$05

	CMP #$01 	; Carry is set to 1
	CMP #$05 	; Zero is set to 1
	CMP #$06 	; Negative is set to 1, zero and carry are set to 0

	LDA #$AA 	; N=1, Z=C=0
	CMP #$22 	; C=N=1, Z=0

	LDA #$00 	; N=0, Z=C=1
	CMP #$FF 	; N=Z=C=0

	NOP
	*/
	write_rom(rom, "a9 05 c9 01 c9 05 c9 06 a9 aa c9 22 a9 00 c9 ff ea");
	9
}

pub fn load_program_cpx(rom: &mut [u8;65_536]) -> u8 {
	/*
	LDA #$05
	STA $0A

	LDX #$04 	; N=C=Z=0
	CPX $0A 	; N=1

	LDX #$FF
	CPX $0A 	; N=C=1 Z=0

	LDX #$05
	CPX $0A 	; Z=C=1 N=0

	NOP
	*/
	write_rom(rom, "a9 05 85 0a a2 04 e4 0a a2 ff e4 0a a2 05 e4 0a ea");
	9
}

pub fn load_program_jsr(rom: &mut [u8;65_536]) -> u8 {
	/*
	JSR $0A0B  	; We push to stack: PC + 2
	- 			; Because we jump, we don't want to execute NOP. Because the PC changed.
	*/
	write_rom(rom, "20 0b 0a");
	1
}
