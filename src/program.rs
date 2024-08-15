use crate::{instruction::Instruction, object::NovaObject};

#[derive(Default)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub immutables: Vec<NovaObject>,
    /// mapping instruction to lines.
    /// vec of tuples (line_number, min_instruction_number, file_name)
    pub lines: Vec<(usize, usize, String)>
}

