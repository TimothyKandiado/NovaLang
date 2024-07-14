use std::fmt::Display;

use crate::object::RegisterValueKind;

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
    /// Local offset
    RLO,
    /// Error tracker
    RERR,
    /// Program Counter
    RPC,
    /// Conditionals
    RCND,
    /// Return
    RRTN,
    /// Max number of general registers / also stores number of local variables in called function
    RMax = 16,
}

#[derive(Debug, Clone, Copy)]
pub struct Register {
    pub kind: RegisterValueKind,
    pub value: u64,
}

impl Register {
    pub fn new(kind: RegisterValueKind, value: u64) -> Self {
        Self { kind, value }
    }

    pub fn empty() -> Self {
        Self {
            kind: RegisterValueKind::None,
            value: 0,
        }
    }
}

impl Default for Register {
    fn default() -> Self {
        Self {
            kind: RegisterValueKind::None,
            value: Default::default(),
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let description = match self.kind {
            RegisterValueKind::Float64 => {
                format!("{:<10} : {:>10}", "Float64", f64::from_bits(self.value))
            }
            RegisterValueKind::Bool => {
                format!("{:<10} : {:>10}", "Bool", self.value == 1)
            }

            RegisterValueKind::None => format!("{:<10}", "None"),
            RegisterValueKind::MemAddress => format!(
                "{:<10} : {:>#10x} | {:>10}",
                "MemAddress", self.value, self.value
            ),
            RegisterValueKind::ImmAddress => format!(
                "{:<10} : {:>#10x} | {:>10}",
                "ImmAddress", self.value, self.value
            ),
        };

        write!(f, "{}", description)
    }
}
