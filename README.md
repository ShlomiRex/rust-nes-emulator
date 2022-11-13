# Rust NES Emulator

This is my attempt to create basic NES emulator in Rust. 

My goal is to not to look at other's code, but to understand on the high-level the NES architecture and create it in Rust.

I intend to use SDL2 for rendering.

# How to get SDL2 working (for developers only)

1. Download SDL2 [here](https://www.libsdl.org/). The file should be called something like: `SDL2-devel-2.24.2-VC.zip`

2. Copy all `.lib` files from: `C:\Users\Shlomi\Downloads\SDL2-devel-2.24.2-VC\SDL2-2.24.2\lib\x64` the zip to rustup toolchain: `C:\Users\Shlomi\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\x86_64-pc-windows-msvc\lib`. Notice I'm using nightly rust channel, not stable.

4. Copy the `SDL2.dll` file to this project root (where cargo, readme is).

5. You can now work with SDL2 and rust nightly.

# Note: nightly rust channel

Currently, the CPU is quite complex. It uses binary arithmetic with multiple integer types. Its just a requirement.

Because of that, and limited time, I chosen to go with nightly rust channel, because it has the features to work with mixed integer types.

Specifically: `mixed_integer_ops`.

So you must use nightly rust. To do so:

`rustup install nightly`

# Resources

The most used resorces:

- Online emulator for quick testing of the CPU: [here](https://skilldrick.github.io/easy6502/#first-program)
- Best CPU instructions summary: [here](https://www.masswerk.at/6502/6502_instruction_set.html#CPX)

Others:

- CPU Registers: [wiki](https://en.wikipedia.org/wiki/MOS_Technology_6502#Registers)
- R6500 Microchip datasheet: [datasheet](https://datasheetspdf.com/pdf-file/527507/Rockwell/R6502/1)
- Complete NES Emulator from scratch: [YouTube](https://www.youtube.com/watch?v=F8kx56OZQhg)
- NES References guide (`nesdev.org`): [wiki](https://www.nesdev.org/wiki/NES_reference_guide)
- yizhang82's blog: [blog](https://yizhang82.dev/nes-emu-overview)
- A blog/website that is no longer maintained but useful (using Wayback machine): [here](https://web.archive.org/web/20210909190432/http://www.obelisk.me.uk/6502/)
- Another youtuber reading the architecture: [YouTube](https://www.youtube.com/watch?v=qJgsuQoy9bc)
- Blog explains 6502 addressing mode: [Emulator101](http://www.emulator101.com/6502-addressing-modes.html#:~:text=The%206502%20has%20the%20ability,to%20the%20address%20being%20accessed.&text=This%20addressing%20mode%20makes%20the,register%20to%20an%20absolute%20address.)
- Introduction to 6502 assembly: [here](https://famicom.party/book/05-6502assembly/)
- Basic NES program that actually works: [here](https://famicom.party/book/03-gettingstarted/)