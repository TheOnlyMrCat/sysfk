#![feature(asm, vec_into_raw_parts)]

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

fn main() {
    let filename = std::env::args().nth(1).expect("No filename provided");
    let file = std::fs::read_to_string(filename).expect("Could not read file");
    let sysfk = parser::parse(&mut file.chars());

    let registers = Box::new(UnsafeCell::new(Registers::default()));
    let (alloc, len, cap) = vec![0; 32768].into_raw_parts();

    let registers_ptr = registers.get();
    unsafe {
        alloc.cast::<*mut Registers>().write_unaligned(registers_ptr);
    }

    let mut stack = vec![alloc];
    run(&sysfk, &registers, &mut stack);

    unsafe { drop(Vec::from_raw_parts(alloc, len, cap)); }
}

fn run(instructions: &[Instruction], registers: &UnsafeCell<Registers>, stack: &mut Vec<*mut u8>) {
    for instruction in instructions {
        match instruction {
            Instruction::Loop(loop_instructions) => {
                while {
                    let ptr = stack.last().unwrap();
                    let value = unsafe { ptr.read() };
                    value != 0
                } {
                    run(loop_instructions, registers, stack);
                }
            },
            Instruction::IncrementPointer => {
                let ptr = stack.last_mut().unwrap();
                *ptr = unsafe { ptr.add(1) };
            },
            Instruction::DecrementPointer => {
                let ptr = stack.last_mut().unwrap();
                *ptr = unsafe { ptr.sub(1) };
            },
            Instruction::IncrementValue => {
                let ptr = *stack.last().unwrap();
                unsafe { *ptr = (*ptr).wrapping_add(1) };
            },
            Instruction::DecrementValue => {
                let ptr = *stack.last().unwrap();
                unsafe { *ptr = (*ptr).wrapping_sub(1) };
            },
            Instruction::EnterPointer => {
                let ptr = *stack.last().unwrap();
                stack.push(unsafe { ptr.cast::<*mut u8>().read_unaligned() });
            },
            Instruction::ExitPointer => {
                stack.pop();
                if stack.is_empty() {
                    return;
                }
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
