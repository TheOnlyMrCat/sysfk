pub enum Instruction {
    Loop(Vec<Instruction>), // []
    IncrementPointer, // >
    DecrementPointer, // <
    IncrementValue, // +
    DecrementValue, // -
    Syscall, // .
    LoadPointer, // ,
    EnterPointer, // |
    ExitPointer, // ^
}

pub fn parse(input: &mut dyn Iterator<Item = char>) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    loop {
        match input.next() {
            Some('[') => {
                let inner = parse(input);
                instructions.push(Instruction::Loop(inner));
            }
            Some(']') => break,
            Some('>') => instructions.push(Instruction::IncrementPointer),
            Some('<') => instructions.push(Instruction::DecrementPointer),
            Some('+') => instructions.push(Instruction::IncrementValue),
            Some('-') => instructions.push(Instruction::DecrementValue),
            Some('.') => instructions.push(Instruction::Syscall),
            Some(',') => instructions.push(Instruction::LoadPointer),
            Some('|') => instructions.push(Instruction::EnterPointer),
            Some('^') => instructions.push(Instruction::ExitPointer),
            Some(_) => continue,
            None => break,
        }
    }
    instructions
}