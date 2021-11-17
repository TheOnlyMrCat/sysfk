# Sysfk: The useful brainfuck derivative

Everyone always goes on about how brainfuck is turing complete, but show me an actual useful program written in it.
One that, perhaps, has random numbers in it, or has a proper GUI, even a TUI?

There's my point. Now, here's my solution: Introducing `sysfk`.

Sysfk extends the brainfuck language you know and love by making it possible to run actual systemcalls. It uses the same
brainfuck commands except for the following differences:
- The `.` command executes the `syscall` assembly instruction
- The `,` command stores a pointer to the current memory cell in the current memory cell (and the next few, since pointers are large)
- The new `|` command goes to a sub-tape pointed to by the current memory cell (and the next few)
- The new `^` command exits from a sub-tape (or exits the program, if in the base tape)

Let's elaborate on that a bit more.

## Syscalls and registers

Obviously, the `syscall` assembly instruction requires the registers to be filled with particular numbers. This is accommodated
for by a specific memory region for syscall registers. The base memory tape is filled in with a pointer to this region when
the program starts executing, and the region is initially zeroed. This region is copied into the general-purpose registers
when the `.` command is executed.

The region contains 8 bytes for each of the following registers, in order:
- `rax`
- `rcx`
- `rdx`
- `rsi`
- `rdi`
- `r8`
- `r9`
- `r10`
- `r11`
- `r12`
- `r13`
- `r14`
- `r15`

`rbx`, `rsp`, and `rbp` are not used.
