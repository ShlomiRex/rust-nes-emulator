Taken from a blog:
https://taywee.github.io/NerdyNights/nerdynights/asmfirstapp.html

Scroll down to see instructions:




3.6.8. Putting It All Together
Download and unzip the master.zip sample files. This lesson is in background. All the code above is in the background.asm file. Make sure that file, mario.chr, and background.bat is in the same folder as NESASM3, then double click on background.bat. That will run NESASM3 and should produce background.nes. Run that NES file in FCEUXD SP to see your background color! Edit background.asm to change the intensity bits 7-5 to make the background red or green.

You can start the Debug… from the Tools menu in FCEUXD SP to watch your code run. Hit the Step Into button, choose Reset from the NES menu, then keep hitting Step Into to run one instruction at a time. On the left is the memory address, next is the hex opcode that the 6502 is actually running. This will be between one and three bytes. After that is the code you wrote, with the comments taken out and labels translated to addresses. The top line is the instruction that is going to run next. So far there isn’t much code, but the debugger will be very helpful later.