use crate::{
    object::{NovaObject, RegisterValueKind},
    register::Register,
};

#[inline(always)]
pub fn add_str_num(string1: &NovaObject, number: Register) -> NovaObject {
    let mut string1 = string1.to_string();

    let string2 = match number.kind {
        RegisterValueKind::Int64 => {
            let value = number.value as i64;
            value.to_string()
        }
        RegisterValueKind::Float64 => {
            let value = f64::from_bits(number.value);
            value.to_string()
        }

        _ => "".to_string(),
    };

    string1.push_str(&string2);
    NovaObject::String(Box::new(string1))
}


#[inline(always)]
pub fn add_num_str(number: Register, string: &NovaObject) -> NovaObject {
    let string2 = string.to_string();

    let mut string1 = match number.kind {
        RegisterValueKind::Int64 => {
            let value = number.value as i64;
            value.to_string()
        }
        RegisterValueKind::Float64 => {
            let value = f64::from_bits(number.value);
            value.to_string()
        }

        _ => "".to_string(),
    };

    string1.push_str(&string2);

    NovaObject::String(Box::new(string1))
}
