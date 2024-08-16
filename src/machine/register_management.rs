use crate::{bytecode::OpCode, instruction::{instruction_decoder, Instruction}, object::{NovaObject, RegisterValueKind}, register::{Register, RegisterID}};

use super::{memory_management::load_object_from_memory, program_management::emit_error_with_message};

pub fn move_register(registers: &mut [Register] , instruction: Instruction) {
    let destination = instruction_decoder::decode_destination_register(instruction);
    let source = instruction_decoder::decode_source_register_1(instruction);

    let value = get_register(registers, source);
    set_value_in_register(registers, destination, value);
}

/// Clear the temporary value registers
#[inline(always)]
pub fn clear_registers(registers: &mut [Register],) {
    for index in 1..RegisterID::R15 as usize {
        registers[index] = Register::empty();
    }
}

#[inline(always)]
pub fn compare_registers(registers: &mut [Register], memory: &mut Vec<NovaObject>, immutables: &[NovaObject], op: OpCode, first: Register, second: Register) -> bool {
    match op {
        OpCode::Less => {
            if first.kind.is_float64() && second.kind.is_float64() {
                let first = f64::from_bits(first.value);
                let second = f64::from_bits(second.value);
                return first < second;
            }

            if first.kind.is_int64() && second.kind.is_int64() {
                let first = first.value as i64;
                let second = second.value as i64;
                return first < second;
            }

            if first.kind.is_int64() && second.kind.is_float64() {
                let first = first.value as i64;
                let second = f64::from_bits(second.value);
                return (first as f64) < second;
            }

            if first.kind.is_float64() && second.kind.is_int64() {
                let first = f64::from_bits(first.value);
                let second = second.value as i64;
                return first < (second as f64);
            }

            if first.kind.is_mem_address() && second.kind.is_mem_address() {
                let first = load_object_from_memory(memory, first.value);
                let second = load_object_from_memory(memory, second.value);

                return first < second;
            }

            if first.kind.is_imm_address() && second.kind.is_imm_address() {
                let first = &immutables[first.value as usize];
                let second = &immutables[second.value as usize];

                return first < second;
            }

            if first.kind.is_imm_address() && second.kind.is_mem_address() {
                let first = &immutables[first.value as usize];
                let second = load_object_from_memory(memory, second.value);

                return first < second;
            }

            if first.kind.is_mem_address() && second.kind.is_imm_address() {
                let first = load_object_from_memory(memory, first.value);
                let second = &immutables[second.value as usize];

                return first < second;
            }
        }

        OpCode::LessEqual => {
            if first.kind.is_float64() && second.kind.is_float64() {
                let first = f64::from_bits(first.value);
                let second = f64::from_bits(second.value);
                return first <= second;
            }

            if first.kind.is_int64() && second.kind.is_int64() {
                let first = first.value as i64;
                let second = second.value as i64;
                return first <= second;
            }

            if first.kind.is_int64() && second.kind.is_float64() {
                let first = first.value as i64;
                let second = f64::from_bits(second.value);
                return (first as f64) <= second;
            }

            if first.kind.is_float64() && second.kind.is_int64() {
                let first = f64::from_bits(first.value);
                let second = second.value as i64;
                return first <= (second as f64);
            }

            if first.kind.is_mem_address() && second.kind.is_mem_address() {
                let first = load_object_from_memory(memory, first.value);
                let second = load_object_from_memory(memory, second.value);

                return first <= second;
            }

            if first.kind.is_imm_address() && second.kind.is_mem_address() {
                let first = &immutables[first.value as usize];
                let second = load_object_from_memory(memory, second.value);

                return first <= second;
            }

            if first.kind.is_mem_address() && second.kind.is_imm_address() {
                let first = load_object_from_memory(memory, first.value);
                let second = &immutables[second.value as usize];

                return first <= second;
            }
        }

        OpCode::Equal => {

            if first.kind.is_int64() && second.kind.is_float64() {
                let first = first.value as i64;
                let second = f64::from_bits(second.value);
                return (first as f64) == second;
            }

            if first.kind.is_float64() && second.kind.is_int64() {
                let first = f64::from_bits(first.value);
                let second = second.value as i64;
                return first == (second as f64);
            }
            
            if first.kind != second.kind {
                return false;
            }

            if first.kind.is_none() && second.kind.is_none() {
                return true;
            }

            if first.kind.is_float64() && second.kind.is_float64() {
                return first.value == second.value;
            }

            if first.kind.is_int64() && second.kind.is_int64() {
                return first.value == second.value;
            }

            if first.kind.is_mem_address() && second.kind.is_mem_address() {
                let first = load_object_from_memory(memory, first.value);
                let second = load_object_from_memory(memory, second.value);

                return first == second;
            }

            if first.kind.is_imm_address() && second.kind.is_mem_address() {
                let first = &immutables[first.value as usize];
                let second = load_object_from_memory(memory, second.value);

                return first == second;
            }

            if first.kind.is_mem_address() && second.kind.is_imm_address() {
                let first = load_object_from_memory(memory, first.value);
                let second = &immutables[second.value as usize];

                return first == second;
            }
        }

        _ => {
            emit_error_with_message(registers, memory, &format!(
                "Undefined comparison operator {:#x}",
                op as Instruction
            ));
        }
    }

    emit_error_with_message(registers, memory, &format!(
        "cannot compare {:?} to {:?}",
        first.kind, second.kind
    ));

    false
}

#[inline(always)]
pub fn clear_register(registers: &mut [Register], register_id: Instruction) {
    let register = unsafe {
        registers.get_unchecked_mut(register_id as usize)
    };

    register.kind = RegisterValueKind::None;
    register.value = 0;
}

#[inline(always)]
pub fn get_register(registers: &[Register], register_id: Instruction) -> Register {
    unsafe {
        return *registers.get_unchecked(register_id as usize);
    }
}

#[inline(always)]
pub fn set_value_in_register(registers: &mut [Register], register_id: Instruction, value: Register) {
    unsafe {
        let register = registers.get_unchecked_mut(register_id as usize);
        register.kind = value.kind;
        register.value = value.value;
    }
}

#[inline(always)]
pub fn load_f64_to_register(registers: &mut [Register], destination: Instruction, number: f64) {
    let number = number.to_bits();
    let register = Register::new(RegisterValueKind::Float64, number);
    set_value_in_register(registers, destination, register);
}

#[inline(always)]
pub fn load_i64_to_register(registers: &mut [Register], destination: Instruction, number: i64) {
    let number = number as u64;
    let register = Register::new(RegisterValueKind::Int64, number);
    set_value_in_register(registers, destination, register);
}

#[inline(always)]
pub fn load_memory_address_to_register(registers: &mut [Register], destination: Instruction, address: Instruction) {
    let value = Register::new(RegisterValueKind::MemAddress, address as u64);
    set_value_in_register(registers, destination, value);
}

#[inline(always)]
pub fn package_register_into_nova_object(registers: &mut [Register], memory: &[NovaObject], immutables: &[NovaObject], register_address: Instruction) -> NovaObject {
    let register = get_register(registers, register_address);

    let value = match register.kind {
        RegisterValueKind::Int64 => NovaObject::Int64(register.value as i64),
        RegisterValueKind::Float64 => NovaObject::Float64(f64::from_bits(register.value)),
        RegisterValueKind::None => NovaObject::None,
        RegisterValueKind::MemAddress => load_object_from_memory(memory, register.value).clone(),
        RegisterValueKind::ImmAddress => immutables[register.value as usize].clone(),
        RegisterValueKind::Bool => todo!(),
        RegisterValueKind::NovaFunctionID(_) => todo!()
    };

    value
}

#[inline(always)]
pub fn is_truthy(register: Register) -> bool {
    match register.kind {
        RegisterValueKind::None => false,
        RegisterValueKind::Int64 => true,
        RegisterValueKind::Float64 => true,
        RegisterValueKind::Bool => register.value == 1,
        RegisterValueKind::MemAddress => true,
        RegisterValueKind::ImmAddress => true,
        RegisterValueKind::NovaFunctionID(_) => true,
    }
}