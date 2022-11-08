# Rust NES Emulator

This is my attempt to create basic NES emulator in Rust. 

My goal is to not to look at other's code, but to understand on the high-level the NES architecture and create it in Rust.

I'm also will be using minimal crates (libraries), so its easy to understand. (Rendering libraries are a must, but bitwise crates are not needed)

I intend to use SDL2 for rendering.

# Resources

- CPU Registers: [wiki](https://en.wikipedia.org/wiki/MOS_Technology_6502#Registers)
- R6500 Microchip datasheet: [datasheet](https://datasheetspdf.com/pdf-file/527507/Rockwell/R6502/1)
- Complete NES Emulator from scratch: [YouTube](https://www.youtube.com/watch?v=F8kx56OZQhg)
- NES References guide (`nesdev.org`): [wiki](https://www.nesdev.org/wiki/NES_reference_guide)
- yizhang82's blog: [blog](https://yizhang82.dev/nes-emu-overview)
- A blog/website that is no longer maintained but useful (using Wayback machine): [here](https://web.archive.org/web/20210909190432/http://www.obelisk.me.uk/6502/)
- Another youtuber reading the architecture: [YouTube](https://www.youtube.com/watch?v=qJgsuQoy9bc)
- Blog explains 6502 addressing mode: [Emulator101](http://www.emulator101.com/6502-addressing-modes.html#:~:text=The%206502%20has%20the%20ability,to%20the%20address%20being%20accessed.&text=This%20addressing%20mode%20makes%20the,register%20to%20an%20absolute%20address.)