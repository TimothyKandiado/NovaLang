use std::collections::HashMap;

use crate::instruction::Instruction;

pub type ValueID = String;
pub type BaseNumber = f32;
pub type MappedMemory = HashMap<ValueID, Instruction>;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum NovaObject {
    None,
    Bool(bool),
    Number(BaseNumber),
    String(Box<String>),
}

#[derive(Debug, Clone, Copy)]
pub enum RegisterValueKind {
    None,
    /// Float32 value
    Float32,
    /// Index of object in memory array
    MemAddress,
    /// Index of object in immutables array
    ImmAddress,
}

impl RegisterValueKind {
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    #[inline(always)]
    pub fn is_float32(&self) -> bool {
        matches!(self, Self::Float32)
    }

    #[inline(always)]
    pub fn is_mem_address(&self) -> bool {
        matches!(self, Self::MemAddress)
    }

    #[inline(always)]
    pub fn is_imm_address(&self) -> bool {
        matches!(self, Self::ImmAddress)
    }
}
