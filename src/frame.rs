use crate::instruction::Instruction;

pub struct Frame {
    pub return_address: Instruction,
    pub local_offset: Instruction,
    pub is_main: bool,
}

impl Frame {
    #[inline(always)]
    pub fn new(return_address: Instruction, local_offset: Instruction, is_main: bool) -> Self {
        Self {
            return_address,
            local_offset,
            is_main,
        }
    }

    #[inline(always)]
    pub fn main() -> Self {
        Self::new(0, 0, true)
    }
}
