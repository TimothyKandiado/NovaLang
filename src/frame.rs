use crate::register::{Register, RegisterID};

#[derive(Debug, Clone)]
pub struct Frame {
    pub is_main: bool,
    pub registers: [Register; RegisterID::RMax as usize + 1],
}

impl Frame {
    #[inline(always)]
    pub fn new(
        registers: [Register; RegisterID::RMax as usize + 1],
        is_main: bool,
    ) -> Self {
        Self {
            is_main,
            registers,
        }
    }

    #[inline(always)]
    pub fn empty(is_main: bool) -> Self {
        let registers = [Register::default(); RegisterID::RMax as usize + 1];
        Self::new(registers, is_main)
    }

    #[inline(always)]
    pub fn main() -> Self {
        let registers = [Register::default(); RegisterID::RMax as usize + 1];
        Self::new(registers, true)
    }
}
