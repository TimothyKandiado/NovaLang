use std::{collections::HashMap, fmt::Display};

use crate::instruction::Instruction;

pub type ValueID = String;
pub type BaseNumber = f32;
pub type MappedMemory = HashMap<ValueID, Instruction>;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct NovaFunction {
    pub name: Box<String>,
    pub address: Instruction,
    pub arity: Instruction,
    pub is_method: bool,
    pub number_of_locals: Instruction,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct NativeFunction {
    pub name: String,
    pub function: fn(Vec<NovaObject>) -> Result<NovaObject, String>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum NovaObject {
    None,
    Float32(f32),
    NovaFunction(NovaFunction),
    NativeFunction(NativeFunction),
    String(Box<String>),
}

pub enum NovaCallable<'a> {
    None,
    NovaFunction(&'a NovaFunction),
    NativeFunction(&'a NativeFunction),
}

impl NovaObject {
    pub fn is_none(&self) -> bool {
        matches!(self, NovaObject::None)
    }

    pub fn is_string(&self) -> bool {
        matches!(self, NovaObject::String(_))
    }

    pub fn is_callable(&self) -> bool {
        matches!(self, NovaObject::NovaFunction(_))
    }
}

impl Display for NovaObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NovaObject::None => write!(f, "None"),
            NovaObject::Float32(float) => write!(f, "{}", float),
            NovaObject::String(string) => write!(f, "{}", string),
            NovaObject::NovaFunction(nova_function) => {
                write!(
                    f,
                    "function: {}, parameters: {}",
                    nova_function.name, nova_function.arity
                )
            }
            
            NovaObject::NativeFunction(native_function) => {
                write!(
                    f,
                    "function: {}",
                    native_function.name
                )
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RegisterValueKind {
    None,
    /// Float32 value
    Float32,
    /// Bool
    Bool,
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

impl PartialEq for RegisterValueKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Float32, Self::Float32) => true,
            (Self::Bool, Self::Bool) => true,
            (Self::MemAddress, Self::MemAddress) => true,
            (Self::ImmAddress, Self::ImmAddress) => true,
            _ => false,
        }
    }
}
