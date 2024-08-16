use std::fmt::Display;

use crate::instruction::Instruction;
use rustc_hash::FxHashMap;

pub type ValueID = String;
pub type BaseNumber = f32;
pub type MappedMemory = FxHashMap<ValueID, Instruction>;

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
    Int64(i64),
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
            NovaCallable::None => "None",
        }
    }

    pub fn as_object(&self) -> NovaObject {
        match self {
            NovaCallable::None => NovaObject::None,
            NovaCallable::NativeFunction(function) => {
                NovaObject::NativeFunction((*function).clone())
            }
            NovaCallable::NovaFunction(function) => NovaObject::NovaFunction((*function).clone()),
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
        matches!(
            self,
            NovaObject::NovaFunction(_) | NovaObject::NativeFunction(_)
        )
    }

    pub fn as_callable(&self) -> NovaCallable {
        match self {
            NovaObject::NovaFunction(nova_function) => NovaCallable::NovaFunction(nova_function),
            NovaObject::NativeFunction(native_function) => {
                NovaCallable::NativeFunction(native_function)
            }
            _ => NovaCallable::None,
        }
    }
}

impl Display for NovaObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NovaObject::None => write!(f, "None"),
            NovaObject::Int64(int) => write!(f, "{}", int),
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
                write!(f, "function: {}", native_function.name)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegisterValueKind {
    /// None value
    None,
    /// Int64 valuel
    Int64,
    /// Float64 value
    Float64,
    /// Bool
    Bool,
    /// Index of object in memory array
    MemAddress,
    /// Index of object in immutables array
    ImmAddress,

    NovaFunctionID(NovaFunctionID),
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
    pub fn is_int64(&self) -> bool {
        matches!(self, Self::Int64)
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NovaFunctionID {
    pub value: u32,
}

impl NovaFunctionID {
    pub fn from_nova_function(
        nova_function: &NovaFunction,
        name_address: Instruction,
    ) -> Option<Self> {
        let mut value = 0u32;

        if nova_function.number_of_locals > 32 {
            return None;
        }

        let shifted = (nova_function.number_of_locals as u32) << 27;
        value += shifted;

        if nova_function.arity > 8 {
            return None;
        }

        let shifted = (nova_function.arity as u32) << 24;
        value += shifted;

        let is_method = if nova_function.is_method { 1u32 } else { 0u32 };
        let shifted = is_method << 23;

        value += shifted;

        if name_address > 2u32.pow(20u32) {
            return None;
        }

        value += name_address;

        Some(Self { value })
    }

    pub fn to_labelled(&self) -> NovaFunctionIDLabelled {
        let mut value = self.value;
        let name_address = value & 0xfffff;
        value = value >> 23;
        let is_method = value & 0x1;
        let is_method = is_method == 1;
        value = value >> 1;
        let arity = value & 0b111;
        value = value >> 3;
        let number_of_locals = value;

        NovaFunctionIDLabelled {
            name_address,
            arity,
            number_of_locals,
            is_method,
        }
    }
}

pub struct NovaFunctionIDLabelled {
    pub name_address: u32,
    pub arity: u32,
    pub number_of_locals: u32,
    pub is_method: bool,
}

#[cfg(test)]
mod tests {
    use super::{NovaFunction, NovaFunctionID};

    #[test]
    fn test_nova_function_id_serialization() {
        let novafunction = NovaFunction {
            name: Box::new(String::from("Hello")),
            arity: 4,
            address: 50,
            is_method: false,
            number_of_locals: 20,
        };

        let name_address = 4444;
        let nova_function_id =
            NovaFunctionID::from_nova_function(&novafunction, name_address).unwrap();
        let labelled = nova_function_id.to_labelled();

        assert_eq!(novafunction.arity, labelled.arity);
        assert_eq!(novafunction.number_of_locals, labelled.number_of_locals);
        assert_eq!(novafunction.is_method, labelled.is_method);
        assert_eq!(name_address, labelled.name_address);
    }
}
