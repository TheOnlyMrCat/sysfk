#![feature(asm)]

use std::cell::UnsafeCell;

use parser::Instruction;

mod parser;

#[derive(Default, Clone, Copy)]
struct Registers {
    rax: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    // rbx, rsp, and rbp are used by LLVM and therefore cannot be used to perform syscalls through asm!()
}

impl std::fmt::Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "rax {:016x} rcx {:016x} rdx {:016x} rsi {:016x}", self.rax, self.rcx, self.rdx, self.rsi)?;
        writeln!(f, "rdi {:016x} r8  {:016x} r9  {:016x} r10 {:016x}", self.rdi, self.r8, self.r9, self.r10)?;
        writeln!(f, "r11 {:016x} r12 {:016x} r13 {:016x} r14 {:016x}", self.r11, self.r12, self.r13, self.r14)?;
        writeln!(f, "r15 {:016x}", self.r15)
    }
}

struct BaseAlloc {
    start: *mut u8,
    len: usize,
    layout: std::alloc::Layout,
}

fn main() {
    let filename = std::env::args().nth(1).expect("No filename provided");
    println!("Executing program at {}", filename);
    let file = std::fs::read_to_string(filename).expect("Could not read file");
    let sysfk = parser::parse(&mut file.chars());

    let registers = Box::new(UnsafeCell::new(Registers::default()));
    let layout = std::alloc::Layout::new::<u8>();
    let mut alloc = BaseAlloc {
        start: unsafe { std::alloc::realloc(std::alloc::alloc(layout), layout, std::mem::size_of::<usize>()) },
        len: std::mem::size_of::<usize>(),
        layout,
    };

    let registers_ptr = registers.get();
    unsafe {
        alloc.start.cast::<*mut Registers>().write_unaligned(registers_ptr);
    }

    let mut stack = vec![alloc.start];
    println!("Begin");
    run(&sysfk, &registers, &mut alloc, &mut stack);
    // Read and dump allocation
    println!("Finished");
    dump_alloc(&registers, &alloc);
}

fn run(instructions: &[Instruction], registers: &UnsafeCell<Registers>, alloc: &mut BaseAlloc, stack: &mut Vec<*mut u8>) {
    for instruction in instructions {
        match instruction {
            Instruction::Loop(loop_instructions) => {
                while {
                    let ptr = stack.last().unwrap();
                    let value = unsafe { ptr.read() };
                    value != 0
                } {
                    run(loop_instructions, registers, alloc, stack);
                }
            },
            Instruction::IncrementPointer => {
                let stack_len = stack.len();
                let ptr = stack.last_mut().unwrap();
                *ptr = unsafe { ptr.add(1) };
                if stack_len == 1 {
                    // Check if need to expand allocation
                    let end = unsafe { alloc.start.add(alloc.len) };
                    if *ptr >= end {
                        // Reallocate with doubled length
                        let new_len = alloc.len * 2;
                        let new_start = unsafe { std::alloc::realloc(alloc.start, alloc.layout, new_len) };

                        // Zero out new memory
                        let mut cur = end;
                        while cur < unsafe { new_start.add(new_len) } {
                            unsafe { *cur = 0; }
                            cur = unsafe { cur.add(1) };
                        }

                        // Update alloc
                        alloc.start = new_start;
                        alloc.len = new_len;
                    }
                }
            },
            Instruction::DecrementPointer => {
                *stack.last_mut().unwrap() = unsafe { stack.last().unwrap().sub(1) };
                //TODO: grow allocation backwards
            },
            Instruction::IncrementValue => {
                let ptr = *stack.last().unwrap();
                unsafe { *ptr = *ptr.wrapping_add(1) };
            },
            Instruction::DecrementValue => {
                let ptr = *stack.last().unwrap();
                unsafe { *ptr = *ptr.wrapping_sub(1) };
            },
            Instruction::EnterPointer => {
                let ptr = *stack.last().unwrap();
                stack.push(unsafe { ptr.cast::<*mut u8>().read_unaligned() });
            },
            Instruction::ExitPointer => {
                stack.pop();
            },
            Instruction::LoadPointer => {
                let ptr = *stack.last().unwrap();
                unsafe { ptr.cast::<*mut u8>().write_unaligned(ptr) };
            },
            Instruction::Syscall => {
                do_syscall(registers);
            },
        }
    }
}

fn do_syscall(cell: &UnsafeCell<Registers>) {
    let registers = cell.get();
    unsafe {
        asm!(
            "syscall",
            inout("rax") (*registers).rax,
            inout("rcx") (*registers).rcx,
            inout("rdx") (*registers).rdx,
            inout("rsi") (*registers).rsi,
            inout("rdi") (*registers).rdi,
            inout("r8") (*registers).r8,
            inout("r9") (*registers).r9,
            inout("r10") (*registers).r10,
            inout("r11") (*registers).r11,
            inout("r12") (*registers).r12,
            inout("r13") (*registers).r13,
            inout("r14") (*registers).r14,
            inout("r15") (*registers).r15,
        );
    }
}

fn dump_alloc(registers: &UnsafeCell<Registers>, alloc: &BaseAlloc) {
    println!("{}", unsafe { *registers.get() });
    for i in 0..alloc.len {
        unsafe {
            print!("{:02x}", alloc.start.add(i).read_unaligned());
        }
        if (i + 1) % 16 == 0 {
            println!();
        }
    }
}