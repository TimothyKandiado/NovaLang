use std::fmt::Display;

use crate::{instruction::Instruction, object::ObjectKind};

pub enum RegisterID {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    /// Error tracker
    RERR,
    /// Program Counter
    RPC,
    /// Conditionals
    RCND,
    /// Control
    RCNT,
    /// Max number of general registers
    RMax = 16,
}

#[derive(Debug, Clone, Copy)]
pub struct Register {
    pub kind: ObjectKind,
    pub value: Instruction,
}

impl Register {
    pub fn new(kind: ObjectKind, value: Instruction) -> Self {
        Self { kind, value }
    }
}

impl Default for Register {
    fn default() -> Self {
        Self {
            kind: ObjectKind::None,
            value: Default::default(),
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let description = match self.kind {
            ObjectKind::Float32 => {
                format!("{:<10} : {:>10}", "Float32", f32::from_bits(self.value))
            }
            ObjectKind::None => format!("{:<10} : {:>#10x}", "None", self.value),
            ObjectKind::MemAddress => format!("{:<10} : {:>#10x}", "MemAddress", self.value),
        };

        write!(f, "{}", description)
    }
}
