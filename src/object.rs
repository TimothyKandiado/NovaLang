use std::collections::HashMap;

pub type ValueID = String;
pub type BaseNumber = f32;
pub type MappedMemory = HashMap<ValueID, NovaObject>;

#[derive(Debug, Clone)]
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
