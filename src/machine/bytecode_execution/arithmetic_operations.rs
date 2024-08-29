use crate::{object::RegisterValueKind, register::Register};

pub enum ArithmeticOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
}

#[inline(always)]
pub fn op_float_float(op: ArithmeticOp, register_1: Register, register_2: Register) -> Register {
    let value_1 = f64::from_bits(register_1.value);
    let value_2 = f64::from_bits(register_2.value);

    let result = match op {
        ArithmeticOp::Add => {
            value_1 + value_2
        }

        ArithmeticOp::Sub => {
            value_1 - value_2
        }

        ArithmeticOp::Mul => {
            value_1 * value_2
        }

        ArithmeticOp::Div => {
            value_1 / value_2
        }
        ArithmeticOp::Pow => {
            value_1.powf(value_2)
        },
        ArithmeticOp::Mod => {
            value_1 % value_2
        },
    };

    let result = result.to_bits();
    Register::new(RegisterValueKind::Float64, result)
}

#[inline(always)]
pub fn op_int_int(op: ArithmeticOp, register_1: Register, register_2: Register) -> Register {
    let value_1 = register_1.value as i64;
    let value_2 = register_2.value as i64;

    let result = match op {
        ArithmeticOp::Add => {
            value_1 + value_2
        }

        ArithmeticOp::Sub => {
            value_1 - value_2
        }

        ArithmeticOp::Mul => {
            value_1 * value_2
        }

        ArithmeticOp::Div => {
            value_1 / value_2
        }
        ArithmeticOp::Pow => {
            ((value_1 as f64).powf(value_2 as f64)) as i64
        },
        ArithmeticOp::Mod => {
            value_1 % value_2
        },
    };

    let result = result as u64;
    Register::new(RegisterValueKind::Float64, result)
}

#[inline(always)]
pub fn op_int_float(op: ArithmeticOp, register_1: Register, register_2: Register) -> Register {
    let value_1 = (register_1.value as i64) as f64;
    let value_2 = f64::from_bits(register_2.value);

    let result = match op {
        ArithmeticOp::Add => {
            value_1 + value_2
        }

        ArithmeticOp::Sub => {
            value_1 - value_2
        }

        ArithmeticOp::Mul => {
            value_1 * value_2
        }

        ArithmeticOp::Div => {
            value_1 / value_2
        }
        ArithmeticOp::Pow => {
            value_1.powf(value_2)
        },
        ArithmeticOp::Mod => {
            value_1 % value_2
        },
    };

    let result = result.to_bits();
    Register::new(RegisterValueKind::Float64, result)
}

#[inline(always)]
pub fn op_float_int(op: ArithmeticOp, register_1: Register, register_2: Register) -> Register {
    let value_1 = f64::from_bits(register_1.value);
    let value_2 = (register_2.value as i64) as f64;

    let result = match op {
        ArithmeticOp::Add => {
            value_1 + value_2
        }

        ArithmeticOp::Sub => {
            value_1 - value_2
        }

        ArithmeticOp::Mul => {
            value_1 * value_2
        }

        ArithmeticOp::Div => {
            value_1 / value_2
        }
        ArithmeticOp::Pow => {
            value_1.powf(value_2)
        },
        ArithmeticOp::Mod => {
            value_1 % value_2
        },
    };

    let result = result.to_bits();
    Register::new(RegisterValueKind::Float64, result)
}
