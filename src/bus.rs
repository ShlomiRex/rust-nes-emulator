/// 2 bytes address
pub type Address = u16;

#[allow(non_snake_case)]
pub struct Bus {
	pub RAM: Box<[u8; 65_536]>
}

impl Bus {

}