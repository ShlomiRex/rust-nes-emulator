mod cpu;
mod bus;
mod memory;

// https://web.archive.org/web/20210803073202/http://www.obelisk.me.uk/6502/architecture.html
// Zero page: 0x0000 - 0x00FF : is the focus of a number of special addressing modes that result in shorter (and quicker) instructions or allow indirect access to the memory (256 bytes of memory)
// Stack: 	  0x0100 - 0x01FF : is reserved for the system stack and which cannot be relocated. (256 bytes of stack!)
// Reserved memory: 0xFFFA - 0xFFFF (last 6 bytes) : must be programmed with the addresses of the non-maskable interrupt handler ($FFFA/B), the power on reset location ($FFFC/D) and the BRK/interrupt request handler ($FFFE/F) respectively.

fn main() {
	println!("Hi");

	
}
