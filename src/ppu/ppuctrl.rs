use crate::common::bits;


pub struct PPUCtrl {
    pub flags: u8
}

/*
7  bit  0
---- ----
VPHB SINN
|||| ||||
|||| ||++- Base nametable address
|||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
|||| |+--- VRAM address increment per CPU read/write of PPUDATA
|||| |     (0: add 1, going across; 1: add 32, going down)
|||| +---- Sprite pattern table address for 8x8 sprites
||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
|||+------ Background pattern table address (0: $0000; 1: $1000)
||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels – see PPU OAM#Byte 1)
|+-------- PPU master/slave select
|          (0: read backdrop from EXT pins; 1: output color on EXT pins)
+--------- Generate an NMI at the start of the
           vertical blanking interval (0: off; 1: on)
*/
pub enum PPUCtrlBits {
	N1,			// nametable (bit 0)
	N2,			// nametable (bit 1)
	I = 2,		// vram
	S,			// pattern table
	B,			// background pattern table
	H,			// sprite size
	P,			// master/slave select
	V			// vertical blanking interval - generate NMI
}

impl PPUCtrl {
	pub fn set(&mut self, bit: PPUCtrlBits, value: bool) {
		bits::set(&mut self.flags, bit as u8, value);
	}

	pub fn get(&self, bit: PPUCtrlBits) -> bool {
		bits::get(self.flags, bit as u8)
	}

	// // TODO: Read where to put msb,lsb in the register. Bit 0 or bit 1?
	// pub fn set_nametable_address(&mut self, addr: u16) {
	// 	let msb = (addr >> 8) as u8;
	// 	let lsb = (addr & 0xFF) as u8;
	// 	// For now, bit 0 is lsb and bit 1 is msb
	// 	bits::set(&mut self.flags, 0, lsb);

	// }
}