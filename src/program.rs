use crate::{instruction::Instruction, object::NovaObject};

#[derive(Default)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub immutables: Vec<NovaObject>,
    /// mapping instruction to lines.
    /// vec of tuples (line_number, min_instruction_number, file_name)
    pub line_definitions: Vec<LineDefinition>
}

#[derive(Debug, Clone)]
pub struct LineDefinition {
    pub last_instruction: usize,
    pub source_line: usize,
    pub source_file: String
}

