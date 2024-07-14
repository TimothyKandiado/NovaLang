use crate::register::{Register, RegisterID};

#[derive(Debug, Clone)]
pub struct Frame {
    pub return_address: u64,
    pub local_offset: u64,
    pub is_main: bool,
    pub registers: [Register; RegisterID::RMax as usize + 1],
}

impl Frame {
    #[inline(always)]
    pub fn new(
        registers: [Register; RegisterID::RMax as usize + 1],
        return_address: u64,
        local_offset: u64,
        is_main: bool,
    ) -> Self {
        Self {
            return_address,
            local_offset,
            is_main,
            registers,
        }
    }

    #[inline(always)]
    pub fn empty(is_main: bool) -> Self {
        let registers = [Register::default(); RegisterID::RMax as usize + 1];
        Self::new(registers, 0, 0, is_main)
    }

    #[inline(always)]
    pub fn main() -> Self {
        let registers = [Register::default(); RegisterID::RMax as usize + 1];
        Self::new(registers, 0, 0, true)
    }
}
