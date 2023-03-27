pub mod bits {
	pub fn set(flags: &mut u8, bit: u8, value: bool) {
		if value {
			*flags |= 1 << bit;
		} else {
			*flags &= !(1 << bit);
		}
	}

	pub fn get(flags: u8, bit: u8) -> bool {
		let index = bit as u8;
		flags & (1 << index) != 0
	}
}

pub type PRG_Bank = [u8; 16_384];
pub type CHR_Bank = [u8; 8_192];