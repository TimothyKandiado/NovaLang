use std::collections::HashMap;

pub type ValueID = String;
pub type BaseNumber = f32;
pub type MappedMemory = HashMap<ValueID, NovaObject>;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum NovaObject {
    None,
    Bool(bool),
    Number(BaseNumber),
    String(Box<String>),
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectKind {
    None,
    Float32,
    MemAddress,
}

impl ObjectKind {
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
}
