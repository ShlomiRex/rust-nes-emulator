# What assembler I'm using

cc65, you can download here: https://sourceforge.net/projects/cc65/files/cc65-snapshot-win32.zip/download

I also added the assembler bin folder to environment path:

![](/resources/readme/Screenshot%202022-11-12%20022747.png)

# How to use assembler

Firstly, the assembley example was taken from [here](https://famicom.party/book/03-gettingstarted/)

Compile the assembly:

`ca65 greenscreen.asm`

Then tell the assembler to link the binary to NES system:

`ld65 greenscreen.o -t nes -o greenscreen.nes`

# How to run

I use the emulator: `FCEUX`

After linking the assembly program with `ld65`, I can open it with the emulator. You should see green screen:

![](/resources/readme/Screenshot%202022-11-12%20023436.png)

# Minimal NES ROM - Run in FCEUX emulator

We can view the only 2 instruction in `minimal.asm` using the debugger:

![](../resources/readme/Screenshot%202022-11-15%20234615.png)