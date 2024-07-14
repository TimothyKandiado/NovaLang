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
    Float64(f64),
    NovaFunction(NovaFunction),
    NativeFunction(NativeFunction),
    String(Box<String>),
}

pub enum NovaCallable<'a> {
    None,
    NovaFunction(&'a NovaFunction),
    NativeFunction(&'a NativeFunction),
}

impl NovaCallable<'_> {
    pub fn get_name(&self) -> &str {
        match self {
            NovaCallable::NovaFunction(function) => function.name.as_str(),
            NovaCallable::NativeFunction(function) => function.name.as_str(),
            NovaCallable::None => "None"
        }
    }

    pub fn as_object(&self) -> NovaObject {
        match self {
            NovaCallable::None => NovaObject::None,
            NovaCallable::NativeFunction(function) => NovaObject::NativeFunction((*function).clone()),
            NovaCallable::NovaFunction(function) => NovaObject::NovaFunction((*function).clone())
        }
    }
}

impl NovaObject {
    pub fn is_none(&self) -> bool {
        matches!(self, NovaObject::None)
    }

    pub fn is_string(&self) -> bool {
        matches!(self, NovaObject::String(_))
    }

    pub fn is_callable(&self) -> bool {
        matches!(self, NovaObject::NovaFunction(_) | NovaObject::NativeFunction(_))
    }

    pub fn as_callable(&self) -> NovaCallable {
        match self {
            NovaObject::NovaFunction(nova_function) => NovaCallable::NovaFunction(nova_function),
            NovaObject::NativeFunction(native_function) => NovaCallable::NativeFunction(native_function),
            _ => NovaCallable::None
        }
    }
}

impl Display for NovaObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NovaObject::None => write!(f, "None"),
            NovaObject::Float64(float) => write!(f, "{}", float),
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegisterValueKind {
    None,
    /// Float32 value
    Float64,
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
    pub fn is_float64(&self) -> bool {
        matches!(self, Self::Float64)
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

