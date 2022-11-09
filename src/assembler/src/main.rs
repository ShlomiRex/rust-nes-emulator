use std::env;
use std::fs;

fn main() {
	let file_path = "C:\\Users\\Shlomi\\Desktop\\Projects\\rust-nes-emulator\\src\\assembler\\helloworld.asm";
    println!("In file {}", file_path);

    let contents = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");

    println!("With text:\n{contents}");
}
