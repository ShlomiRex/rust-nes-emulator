use crate::common::bits;


pub struct PPUMask {
    pub flags: u8
}

/*
7  bit  0
---- ----
BGRs bMmG
|||| ||||
|||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
|||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
|||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
|||| +---- 1: Show background
|||+------ 1: Show sprites
||+------- Emphasize red (green on PAL/Dendy)
|+-------- Emphasize green (red on PAL/Dendy)
+--------- Emphasize blue
*/
pub enum PPUMaskBits {
	GRAYSCALE,			// G
	ShowBackground,		// m
	M,					// M
	BACKGROUND,			// b
	SPRITES,			// s
	R,					// R
	G,					// G
	B					// B
}

impl PPUMask {
	pub fn set(&mut self, bit: PPUMaskBits, value: bool) {
		bits::set(&mut self.flags, bit as u8, value);
	}

	pub fn get(&self, bit: PPUMaskBits) -> bool {
		bits::get(self.flags, bit as u8)
	}
}