.segment "HEADER"
	.byte "NES"
	.byte $1A

.segment "STARTUP"

.segment "VECTORS"

.segment "CHARS"

lda #$AA
ldx #$BB
ldy #$CC
stop:
	NOP
	jmp stop
	