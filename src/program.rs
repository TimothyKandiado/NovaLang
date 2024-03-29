use crate::{instruction::Instruction, object::NovaObject};

pub struct Program {
    pub instructions: Vec<Instruction>,
    pub immutables: Vec<NovaObject>,
}
