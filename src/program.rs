use crate::{instruction::Instruction, object::NovaObject};

#[derive(Default)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub immutables: Vec<NovaObject>,
}

