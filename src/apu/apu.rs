pub struct APU {
	// 0x4000 - 0x4017
	pub registers: [u8; 0x17]
}

impl APU {
	pub fn new() -> Self {
		APU {
			registers: [0;0x17]
		}
	}
}