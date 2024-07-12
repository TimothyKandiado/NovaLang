use crate::{instruction::Instruction, object::NovaObject};

pub struct Program {
    pub instructions: Vec<Instruction>,
    pub immutables: Vec<NovaObject>,
}

impl Default for Program {
    fn default() -> Self {
        Self {
            instructions: Default::default(),
            immutables: Default::default(),
        }
    }
}
