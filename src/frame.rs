use crate::register::{Register, RegisterID};

pub struct Frame {
    pub registers: [Register; RegisterID::RMax as usize + 1],
}