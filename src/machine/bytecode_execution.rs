use crate::{
    bytecode::OpCode,
    instruction::{instruction_decoder, Instruction},
    object::{NovaCallable, NovaFunctionIDLabelled, NovaObject, RegisterValueKind},
    register::{Register, RegisterID},
};

use super::{
    array_copy,
    memory_management::{
        allocate_global, allocate_local_variables, create_global, load_global_value,
        load_object_from_memory, set_global_value, store_object_in_memory,
    },
    program_management::{
        check_error, drop_frame, emit_error_with_message, get_next_instruction, new_frame,
    },
    register_management::{
        clear_register, compare_registers, get_register, is_truthy, load_f64_to_register,
        load_i64_to_register, load_memory_address_to_register, package_register_into_nova_object,
        set_value_in_register,
    },
    VirtualMachineData,
};

#[inline(always)]
pub fn invoke(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;
    let invoke_register = instruction_decoder::decode_source_register_2(instruction);
    let argument_start = instruction_decoder::decode_destination_register(instruction);
    let argument_number = instruction_decoder::decode_source_register_1(instruction);

    let register = get_register(*registers, invoke_register);

    if let RegisterValueKind::NovaFunctionID(nova_function_id) = register.kind {
        let function_address = register.value;
        invoke_nova_function_id_labelled(
            virtual_machine_data,
            nova_function_id.to_labelled(),
            function_address,
            argument_start,
            argument_number,
        );
        return;
    }

    let registers = &mut virtual_machine_data.registers;
    let memory = &mut virtual_machine_data.memory;
    let immutables = &mut virtual_machine_data.immutables;

    if register.kind != RegisterValueKind::MemAddress {
        emit_error_with_message(*registers, *memory, "Function not found");
        return;
    }

    let nova_object = load_object_from_memory(*memory, register.value);

    let callable = match nova_object {
        NovaObject::NovaFunction(nova_function) => NovaCallable::NovaFunction(nova_function),
        NovaObject::NativeFunction(native_function) => {
            NovaCallable::NativeFunction(native_function)
        }
        _ => NovaCallable::None,
    };

    match callable {
        NovaCallable::NovaFunction(function) => {
            let nova_function_id = NovaFunctionIDLabelled {
                name_address: 0,
                arity: function.arity,
                is_method: function.is_method,
                number_of_locals: function.number_of_locals,
            };

            let function_address = function.address;
            invoke_nova_function_id_labelled(
                virtual_machine_data,
                nova_function_id,
                function_address as u64,
                argument_start,
                argument_number,
            );
        }

        NovaCallable::NativeFunction(function) => {
            let mut source_index = argument_start;
            let source_end = argument_start + argument_number;

            let mut arguments = Vec::new();

            while source_index < source_end {
                let object =
                    package_register_into_nova_object(*registers, memory, immutables, source_index);
                arguments.push(object);
                source_index += 1;
            }

            let result = (function.function)(arguments);

            if let Err(error) = result {
                emit_error_with_message(*registers, *memory, &error);
                return;
            }

            let result = result.unwrap();

            match result {
                NovaObject::Float64(value) => {
                    let register = Register::new(RegisterValueKind::Float64, value.to_bits());
                    set_value_in_register(*registers, RegisterID::RRTN as Instruction, register);
                }

                NovaObject::Int64(value) => {
                    let register = Register::new(RegisterValueKind::Int64, value as u64);
                    set_value_in_register(*registers, RegisterID::RRTN as Instruction, register);
                }

                NovaObject::None => {
                    set_value_in_register(
                        *registers,
                        RegisterID::RRTN as Instruction,
                        Register::empty(),
                    );
                }

                _ => {
                    let memory_location = store_object_in_memory(*memory, result);
                    let register =
                        Register::new(RegisterValueKind::MemAddress, memory_location as u64);
                    set_value_in_register(*registers, RegisterID::RRTN as Instruction, register);
                }
            }
        }

        NovaCallable::None => {
            emit_error_with_message(*registers, *memory, "Called a None Value");
        }
    }
}

#[inline(always)]
fn invoke_nova_function_id_labelled(
    virtual_machine_data: &mut VirtualMachineData,
    nova_function_id: NovaFunctionIDLabelled,
    function_address: u64,
    argument_start: u32,
    argument_number: u32,
) {
    let registers = &mut virtual_machine_data.registers;
    let memory = &mut virtual_machine_data.memory;
    let frames = &mut virtual_machine_data.frames;
    let locals = &mut virtual_machine_data.locals;

    let function = nova_function_id;

    if argument_number != function.arity {
        emit_error_with_message(
            *registers,
            *memory,
            &format!(
                "Not enough function arguments.\n{} are required\n{} were provided",
                function.arity, argument_number
            ),
        );
        return;
    }

    let num_locals = function.number_of_locals;
    new_frame(*registers, *frames, *locals, num_locals);
    let old_frame = frames.last().unwrap();

    let source_index = argument_start as usize;
    let source_end = (argument_start + argument_number) as usize;
    let destination_index = 0;
    let length = source_end - source_index;

    array_copy(
        &old_frame.registers,
        source_index,
        *registers,
        destination_index,
        length,
    );

    unsafe {
        registers.get_unchecked_mut(RegisterID::RPC as usize).value = function_address as u64;
    }
}

#[inline(always)]
pub fn return_none(_: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;
    let frames = &mut virtual_machine_data.frames;
    let locals = &mut virtual_machine_data.locals;
    let running_state = &mut virtual_machine_data.running;

    set_value_in_register(
        *registers,
        RegisterID::RRTN as Instruction,
        Register::empty(),
    );
    drop_frame(*registers, *frames, *locals, *running_state);
}

#[inline(always)]
pub fn return_val(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;
    let frames = &mut virtual_machine_data.frames;
    let locals = &mut virtual_machine_data.locals;
    let running_state = &mut virtual_machine_data.running;

    let value_source = instruction_decoder::decode_source_register_1(instruction);
    let value_register = get_register(*registers, value_source);

    set_value_in_register(*registers, RegisterID::RRTN as Instruction, value_register);

    drop_frame(*registers, *frames, *locals, *running_state);
}

#[inline(always)]
pub fn load_return(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;

    let destination = instruction_decoder::decode_destination_register(instruction);

    let return_register = unsafe { *registers.get_unchecked(RegisterID::RRTN as usize) };
    set_value_in_register(*registers, destination, return_register);
}

#[inline(always)]
pub fn print(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;
    let memory = &mut virtual_machine_data.memory;
    let immutables = &mut virtual_machine_data.immutables;

    let source = instruction_decoder::decode_source_register_1(instruction);
    let newline = instruction_decoder::decode_destination_register(instruction);

    let register = get_register(*registers, source);

    match register.kind {
        RegisterValueKind::Int64 => {
            print!("{}", register.value as i64)
        }
        RegisterValueKind::Float64 => {
            print!("{}", f64::from_bits(register.value))
        }
        RegisterValueKind::None => {
            print!("None")
        }
        RegisterValueKind::Bool => {
            print!("{}", register.value == 1)
        }
        RegisterValueKind::MemAddress => {
            let address = register.value;
            let object = load_object_from_memory(*memory, address);
            print!("{}", object);
        }

        RegisterValueKind::ImmAddress => {
            let immutable = &immutables[register.value as usize];
            print!("{}", immutable);
        }

        RegisterValueKind::NovaFunctionID(_) => todo!(),
    }
    if newline == 1 {
        println!()
    }
}

#[inline(always)]
pub fn negate(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;
    let memory = &mut virtual_machine_data.memory;

    let source = instruction_decoder::decode_source_register_1(instruction);
    let destination = source; // negate value in place

    let register = get_register(*registers, source);
    if let RegisterValueKind::Float64 = register.kind {
        let value = f64::from_bits(register.value);
        let value = -value;
        let value = value.to_bits();

        let register = Register::new(RegisterValueKind::Float64, value);
        set_value_in_register(*registers, destination, register);
        return;
    }

    emit_error_with_message(*registers, *memory, "Cannot negate non float32 value");
}

#[inline(always)]
pub fn add(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;
    let memory = &mut virtual_machine_data.memory;
    let immutables = &mut virtual_machine_data.immutables;

    let destination_register = instruction_decoder::decode_destination_register(instruction);
    let source_register_1 = instruction_decoder::decode_source_register_1(instruction);
    let source_register_2 = instruction_decoder::decode_source_register_2(instruction);

    let register_1 = get_register(*registers, source_register_1);
    let register_2 = get_register(*registers, source_register_2);

    if let (RegisterValueKind::Float64, RegisterValueKind::Float64) =
        (register_1.kind, register_2.kind)
    {
        let value_1 = f64::from_bits(register_1.value);
        let value_2 = f64::from_bits(register_2.value);

        let sum = value_1 + value_2;
        let sum = sum.to_bits();

        let new_value = Register::new(RegisterValueKind::Float64, sum);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    if let (RegisterValueKind::Int64, RegisterValueKind::Int64) = (register_1.kind, register_2.kind)
    {
        let value_1 = register_1.value as i64;
        let value_2 = register_2.value as i64;

        let result = value_1 + value_2;
        let result = result as u64;

        let new_value = Register::new(RegisterValueKind::Int64, result);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    if let (RegisterValueKind::Float64, RegisterValueKind::Int64) =
        (register_1.kind, register_2.kind)
    {
        let value_1 = f64::from_bits(register_1.value);
        let value_2 = register_2.value as i64;

        let result = value_1 + (value_2 as f64);
        let result = result.to_bits();

        let new_value = Register::new(RegisterValueKind::Float64, result);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    if let (RegisterValueKind::Int64, RegisterValueKind::Float64) =
        (register_1.kind, register_2.kind)
    {
        let value_1 = register_1.value as i64;
        let value_2 = f64::from_bits(register_2.value);

        let result = value_1 as f64 + value_2;
        let result = result.to_bits();

        let new_value = Register::new(RegisterValueKind::Float64, result);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    if let (RegisterValueKind::MemAddress, RegisterValueKind::Float64) =
        (register_1.kind, register_2.kind)
    {
        let object1 = load_object_from_memory(*memory, register_1.value);
        if let NovaObject::String(string) = object1 {
            let value2 = f64::from_bits(register_2.value);
            let value2 = value2.to_string();

            let mut new_value = *string.clone();
            new_value.push_str(&value2);

            let new_object = NovaObject::String(Box::new(new_value));
            let address = store_object_in_memory(*memory, new_object);
            load_memory_address_to_register(*registers, destination_register, address);
            return;
        }
        emit_error_with_message(
            *registers,
            *memory,
            &format!("cannot add {:?} to {:?}", object1, register_2.kind),
        );
    }

    if let (RegisterValueKind::ImmAddress, RegisterValueKind::Float64) =
        (register_1.kind, register_2.kind)
    {
        let object1 = &immutables[register_1.value as usize];
        if let NovaObject::String(string) = object1 {
            let value2 = f64::from_bits(register_2.value);
            let value2 = value2.to_string();

            let mut new_value = *string.clone();
            new_value.push_str(&value2);

            let new_object = NovaObject::String(Box::new(new_value));
            let address = store_object_in_memory(*memory, new_object);
            load_memory_address_to_register(*registers, destination_register, address);
            return;
        }
        emit_error_with_message(
            *registers,
            *memory,
            &format!("cannot add {:?} to {:?}", object1, register_2.kind),
        );
    }

    if let (RegisterValueKind::Float64, RegisterValueKind::MemAddress) =
        (register_1.kind, register_2.kind)
    {
        let object2 = load_object_from_memory(*memory, register_2.value);
        if let NovaObject::String(string) = object2 {
            let value1 = f64::from_bits(register_1.value);
            let value1 = value1.to_string();

            let mut new_value = value1;
            new_value.push_str(string);

            let new_object = NovaObject::String(Box::new(new_value));
            let address = store_object_in_memory(*memory, new_object);
            load_memory_address_to_register(*registers, destination_register, address);
            return;
        }
        emit_error_with_message(
            *registers,
            *memory,
            &format!("cannot add {:?} to {:?}", register_1.kind, object2),
        );
    }

    if let (RegisterValueKind::Float64, RegisterValueKind::ImmAddress) =
        (register_1.kind, register_2.kind)
    {
        let object2 = &immutables[register_2.value as usize];
        if let NovaObject::String(string) = object2 {
            let value1 = f64::from_bits(register_1.value);
            let value1 = value1.to_string();

            let mut new_value = value1;
            new_value.push_str(string);

            let new_object = NovaObject::String(Box::new(new_value));
            let address = store_object_in_memory(*memory, new_object);
            load_memory_address_to_register(*registers, destination_register, address);
            return;
        }
        emit_error_with_message(
            *registers,
            *memory,
            &format!("cannot add {:?} to {:?}", register_1.kind, object2),
        );
    }

    if let (RegisterValueKind::ImmAddress, RegisterValueKind::ImmAddress) =
        (register_1.kind, register_2.kind)
    {
        let object1 = &immutables[register_1.value as usize];
        let object2 = &immutables[register_2.value as usize];

        if let (NovaObject::String(string1), NovaObject::String(string2)) = (object1, object2) {

            let mut new_value = string1.as_str().to_string();
            new_value.push_str(&string2);

            let new_object = NovaObject::String(Box::new(new_value));
            let address = store_object_in_memory(*memory, new_object);
            load_memory_address_to_register(*registers, destination_register, address);
            return;
        }
        emit_error_with_message(
            *registers,
            *memory,
            &format!("cannot add {:?} to {:?}", register_1.kind, object2),
        );
    }

    if let (RegisterValueKind::MemAddress, RegisterValueKind::MemAddress) =
        (register_1.kind, register_2.kind)
    {
        let object1 = &memory[register_1.value as usize];
        let object2 = &memory[register_2.value as usize];

        if let (NovaObject::String(string1), NovaObject::String(string2)) = (object1, object2) {

            let mut new_value = string1.as_str().to_string();
            new_value.push_str(&string2);

            let new_object = NovaObject::String(Box::new(new_value));
            let address = store_object_in_memory(*memory, new_object);
            load_memory_address_to_register(*registers, destination_register, address);
            return;
        }
        emit_error_with_message(
            *registers,
            *memory,
            &format!("cannot add {:?} to {:?}", register_1.kind, object2),
        );
    }

    emit_error_with_message(
        *registers,
        *memory,
        &format!("cannot add {:?} to {:?}", register_1.kind, register_2.kind),
    )

}

#[inline(always)]
pub fn sub(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;
    let memory = &mut virtual_machine_data.memory;

    let destination_register = instruction_decoder::decode_destination_register(instruction);
    let source_register_1 = instruction_decoder::decode_source_register_1(instruction);
    let source_register_2 = instruction_decoder::decode_source_register_2(instruction);

    let register_1 = get_register(*registers, source_register_1);
    let register_2 = get_register(*registers, source_register_2);

    if let (RegisterValueKind::Float64, RegisterValueKind::Float64) =
        (register_1.kind, register_2.kind)
    {
        let value_1 = f64::from_bits(register_1.value);
        let value_2 = f64::from_bits(register_2.value);

        let sub = value_1 - value_2;
        let sub = sub.to_bits();

        let new_value = Register::new(register_1.kind, sub);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    if let (RegisterValueKind::Int64, RegisterValueKind::Int64) = (register_1.kind, register_2.kind)
    {
        let value_1 = register_1.value as i64;
        let value_2 = register_2.value as i64;

        let result = value_1 - value_2;
        let result = result as u64;

        let new_value = Register::new(RegisterValueKind::Int64, result);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    if let (RegisterValueKind::Float64, RegisterValueKind::Int64) =
        (register_1.kind, register_2.kind)
    {
        let value_1 = f64::from_bits(register_1.value);
        let value_2 = register_2.value as i64;

        let result = value_1 - (value_2 as f64);
        let result = result.to_bits();

        let new_value = Register::new(RegisterValueKind::Float64, result);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    if let (RegisterValueKind::Int64, RegisterValueKind::Float64) =
        (register_1.kind, register_2.kind)
    {
        let value_1 = register_1.value as i64;
        let value_2 = f64::from_bits(register_2.value);

        let result = value_1 as f64 - value_2;
        let result = result.to_bits();

        let new_value = Register::new(RegisterValueKind::Float64, result);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    emit_error_with_message(
        *registers,
        *memory,
        &format!(
            "cannot subtract {:?} by {:?}",
            register_1.kind, register_2.kind
        ),
    )
}

#[inline(always)]
pub fn mul(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;
    let memory = &mut virtual_machine_data.memory;

    let destination_register = instruction_decoder::decode_destination_register(instruction);
    let source_register_1 = instruction_decoder::decode_source_register_1(instruction);
    let source_register_2 = instruction_decoder::decode_source_register_2(instruction);

    let register_1 = get_register(*registers, source_register_1);
    let register_2 = get_register(*registers, source_register_2);

    if let (RegisterValueKind::Float64, RegisterValueKind::Float64) =
        (register_1.kind, register_2.kind)
    {
        let value_1 = f64::from_bits(register_1.value);
        let value_2 = f64::from_bits(register_2.value);

        let mul = value_1 * value_2;
        let mul = mul.to_bits();

        let new_value = Register::new(register_1.kind, mul);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    if let (RegisterValueKind::Int64, RegisterValueKind::Int64) = (register_1.kind, register_2.kind)
    {
        let value_1 = register_1.value as i64;
        let value_2 = register_2.value as i64;

        let result = value_1 * value_2;
        let result = result as u64;

        let new_value = Register::new(RegisterValueKind::Int64, result);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    if let (RegisterValueKind::Float64, RegisterValueKind::Int64) =
        (register_1.kind, register_2.kind)
    {
        let value_1 = f64::from_bits(register_1.value);
        let value_2 = register_2.value as i64;

        let result = value_1 * (value_2 as f64);
        let result = result.to_bits();

        let new_value = Register::new(RegisterValueKind::Float64, result);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    if let (RegisterValueKind::Int64, RegisterValueKind::Float64) =
        (register_1.kind, register_2.kind)
    {
        let value_1 = register_1.value as i64;
        let value_2 = f64::from_bits(register_2.value);

        let result = value_1 as f64 * value_2;
        let result = result.to_bits();

        let new_value = Register::new(RegisterValueKind::Float64, result);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    emit_error_with_message(
        *registers,
        *memory,
        &format!(
            "cannot multiply {:?} by {:?}",
            register_1.kind, register_2.kind
        ),
    )
}

#[inline(always)]
pub fn div(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;
    let memory = &mut virtual_machine_data.memory;

    let destination_register = instruction_decoder::decode_destination_register(instruction);
    let source_register_1 = instruction_decoder::decode_source_register_1(instruction);
    let source_register_2 = instruction_decoder::decode_source_register_2(instruction);

    let register_1 = get_register(*registers, source_register_1);
    let register_2 = get_register(*registers, source_register_2);

    if let (RegisterValueKind::Float64, RegisterValueKind::Float64) =
        (register_1.kind, register_2.kind)
    {
        let value_1 = f64::from_bits(register_1.value);
        let value_2 = f64::from_bits(register_2.value);

        let div = value_1 / value_2;
        let div = div.to_bits();

        let new_value = Register::new(register_1.kind, div);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    if let (RegisterValueKind::Int64, RegisterValueKind::Int64) = (register_1.kind, register_2.kind)
    {
        let value_1 = register_1.value as i64;
        let value_2 = register_2.value as i64;

        let result = value_1 as f64 / value_2 as f64;
        let result = result.to_bits();

        let new_value = Register::new(RegisterValueKind::Float64, result);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    if let (RegisterValueKind::Float64, RegisterValueKind::Int64) =
        (register_1.kind, register_2.kind)
    {
        let value_1 = f64::from_bits(register_1.value);
        let value_2 = register_2.value as i64;

        let result = value_1 / (value_2 as f64);
        let result = result.to_bits();

        let new_value = Register::new(RegisterValueKind::Float64, result);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    if let (RegisterValueKind::Int64, RegisterValueKind::Float64) =
        (register_1.kind, register_2.kind)
    {
        let value_1 = register_1.value as i64;
        let value_2 = f64::from_bits(register_2.value);

        let result = value_1 as f64 / value_2;
        let result = result.to_bits();

        let new_value = Register::new(RegisterValueKind::Float64, result);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    emit_error_with_message(
        *registers,
        *memory,
        &format!(
            "cannot divide {:?} by {:?}",
            register_1.kind, register_2.kind
        ),
    )
}

#[inline(always)]
pub fn pow(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;
    let memory = &mut virtual_machine_data.memory;

    let destination_register = instruction_decoder::decode_destination_register(instruction);
    let source_register_1 = instruction_decoder::decode_source_register_1(instruction);
    let source_register_2 = instruction_decoder::decode_source_register_2(instruction);

    let register_1 = get_register(*registers, source_register_1);
    let register_2 = get_register(*registers, source_register_2);

    if let (RegisterValueKind::Float64, RegisterValueKind::Float64) =
        (register_1.kind, register_2.kind)
    {
        let value_1 = f64::from_bits(register_1.value);
        let value_2 = f64::from_bits(register_2.value);

        let pow = value_1.powf(value_2);
        let pow = pow.to_bits();

        let new_value = Register::new(register_1.kind, pow);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    if let (RegisterValueKind::Int64, RegisterValueKind::Int64) = (register_1.kind, register_2.kind)
    {
        let value_1 = register_1.value as i64;
        let value_2 = register_2.value as i64;

        let result = (value_1 as f64).powf(value_2 as f64);
        let result = result.to_bits();

        let new_value = Register::new(RegisterValueKind::Float64, result);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    if let (RegisterValueKind::Float64, RegisterValueKind::Int64) =
        (register_1.kind, register_2.kind)
    {
        let value_1 = f64::from_bits(register_1.value);
        let value_2 = register_2.value as i64;

        let result = value_1.powf(value_2 as f64);
        let result = result.to_bits();

        let new_value = Register::new(RegisterValueKind::Float64, result);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    if let (RegisterValueKind::Int64, RegisterValueKind::Float64) =
        (register_1.kind, register_2.kind)
    {
        let value_1 = register_1.value as i64;
        let value_2 = f64::from_bits(register_2.value);

        let result = (value_1 as f64).powf(value_2);
        let result = result.to_bits();

        let new_value = Register::new(RegisterValueKind::Float64, result);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    emit_error_with_message(
        *registers,
        *memory,
        &format!(
            "cannot calculate power of {:?} to {:?}",
            register_1.kind, register_2.kind
        ),
    )
}

#[inline(always)]
pub fn modulus(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;
    let memory = &mut virtual_machine_data.memory;

    let destination_register = instruction_decoder::decode_destination_register(instruction);
    let source_register_1 = instruction_decoder::decode_source_register_1(instruction);
    let source_register_2 = instruction_decoder::decode_source_register_2(instruction);

    let register_1 = get_register(*registers, source_register_1);
    let register_2 = get_register(*registers, source_register_2);

    if let (RegisterValueKind::Float64, RegisterValueKind::Float64) =
        (register_1.kind, register_2.kind)
    {
        let value_1 = f64::from_bits(register_1.value) as i32;
        let value_2 = f64::from_bits(register_2.value) as i32;

        let modulus = (value_1 % value_2) as i64;
        let modulus = modulus as u64;

        let new_value = Register::new(RegisterValueKind::Int64, modulus);
        set_value_in_register(*registers, destination_register, new_value);
        return;
    }

    emit_error_with_message(
        *registers,
        *memory,
        &format!(
            "cannot find modulus {:?} by {:?}",
            register_1.kind, register_2.kind
        ),
    )
}

#[inline(always)]
/// compares if first register is less than second register
pub fn less(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;
    let memory = &mut virtual_machine_data.memory;
    let immutables = &mut virtual_machine_data.immutables;

    let source1 = instruction_decoder::decode_source_register_1(instruction);
    let source2 = instruction_decoder::decode_source_register_2(instruction);

    let destination = instruction_decoder::decode_destination_register(instruction);

    let register1 = get_register(*registers, source1);
    let register2 = get_register(*registers, source2);

    let less = compare_registers(
        *registers,
        *memory,
        *immutables,
        OpCode::Less,
        register1,
        register2,
    );
    if check_error(*registers) {
        return;
    }

    let register = Register {
        value: if less { 1 } else { 0 },
        kind: RegisterValueKind::Bool,
    };

    set_value_in_register(*registers, destination, register);
}

#[inline(always)]
/// compares if first register is less than or equal to second register
pub fn less_or_equal(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;
    let memory = &mut virtual_machine_data.memory;
    let immutables = &mut virtual_machine_data.immutables;

    let source1 = instruction_decoder::decode_source_register_1(instruction);
    let source2 = instruction_decoder::decode_source_register_2(instruction);

    let destination = instruction_decoder::decode_destination_register(instruction);

    let register1 = get_register(*registers, source1);
    let register2 = get_register(*registers, source2);

    let less = compare_registers(
        *registers,
        *memory,
        *immutables,
        OpCode::LessEqual,
        register1,
        register2,
    );
    if check_error(*registers) {
        return;
    }

    let register = Register {
        value: if less { 1 } else { 0 },
        kind: RegisterValueKind::Bool,
    };

    set_value_in_register(*registers, destination, register);
}

#[inline(always)]
/// compares if first register is less than or equal to second register
pub fn equal(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;
    let memory = &mut virtual_machine_data.memory;
    let immutables = &mut virtual_machine_data.immutables;

    let source1 = instruction_decoder::decode_source_register_1(instruction);
    let source2 = instruction_decoder::decode_source_register_2(instruction);

    let destination = instruction_decoder::decode_destination_register(instruction);

    let register1 = get_register(*registers, source1);
    let register2 = get_register(*registers, source2);

    let equal = compare_registers(
        *registers,
        *memory,
        *immutables,
        OpCode::Equal,
        register1,
        register2,
    );
    if check_error(*registers) {
        return;
    }

    let register = Register {
        value: if equal { 1 } else { 0 },
        kind: RegisterValueKind::Bool,
    };

    set_value_in_register(*registers, destination, register);
}

#[inline(always)]
pub fn not(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;

    let source = instruction_decoder::decode_source_register_1(instruction);
    let mut register = get_register(*registers, source);

    let is_true = is_truthy(register);
    register.value = if is_true { 0 } else { 1 };

    set_value_in_register(*registers, source, register);
}

#[inline(always)]
pub fn jump_if_false(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;
    let instructions = &virtual_machine_data.instructions;

    let source = instruction_decoder::decode_source_register_1(instruction);

    let register = get_register(*registers, source);
    let truthy = is_truthy(register);

    let jump_instruction = get_next_instruction(*registers, instructions);

    if !truthy {
        jump(jump_instruction, virtual_machine_data);
    }
}

#[inline(always)]
pub fn jump(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;

    let offset = instruction_decoder::decode_immutable_address_small(instruction);
    let direction = instruction_decoder::decode_destination_register(instruction);

    if direction == 0 {
        registers[RegisterID::RPC as usize].value -= offset as u64 + 1; // backward jump, add one since the intepreter will automatically add 1 after instruction
    } else {
        registers[RegisterID::RPC as usize].value += offset as u64 - 1; // forward jump, minus one since the intepreter will automatically add 1 after instruction
    }
}

#[inline(always)]
pub fn load_constant_to_register(
    instruction: Instruction,
    virtual_machine_data: &mut VirtualMachineData,
) {
    let registers = &mut virtual_machine_data.registers;

    let destination_register = instruction_decoder::decode_destination_register(instruction);
    let immutable_address = instruction_decoder::decode_immutable_address_small(instruction);

    let register = Register {
        kind: RegisterValueKind::ImmAddress,
        value: immutable_address as u64,
    };

    set_value_in_register(*registers, destination_register, register);
}

#[inline(always)]
pub fn load_float32_to_register(
    instruction: Instruction,
    virtual_machine_data: &mut VirtualMachineData,
) {
    let registers = &mut virtual_machine_data.registers;
    let instructions = &mut virtual_machine_data.instructions;

    let destination_register = instruction_decoder::decode_destination_register(instruction);

    let number = get_next_instruction(*registers, instructions);
    let number = f32::from_bits(number);
    load_f64_to_register(*registers, destination_register, number as f64);
}

#[inline(always)]
pub fn load_float64_to_register(
    instruction: Instruction,
    virtual_machine_data: &mut VirtualMachineData,
) {
    let registers = &mut virtual_machine_data.registers;
    let instructions = &mut virtual_machine_data.instructions;

    let destination_register = instruction_decoder::decode_destination_register(instruction);

    let first_half = get_next_instruction(*registers, instructions);
    let second_half = get_next_instruction(*registers, instructions);

    let number = instruction_decoder::merge_u32s(first_half, second_half);
    let number = f64::from_bits(number);
    load_f64_to_register(*registers, destination_register, number);
}

#[inline(always)]
pub fn load_int32_to_register(
    instruction: Instruction,
    virtual_machine_data: &mut VirtualMachineData,
) {
    let registers = &mut virtual_machine_data.registers;
    let instructions = &mut virtual_machine_data.instructions;

    let destination_register = instruction_decoder::decode_destination_register(instruction);

    let number = get_next_instruction(*registers, *instructions);
    let number = number as i32;
    load_i64_to_register(*registers, destination_register, number as i64);
}

#[inline(always)]
pub fn load_int64_to_register(
    instruction: Instruction,
    virtual_machine_data: &mut VirtualMachineData,
) {
    let registers = &mut virtual_machine_data.registers;
    let instructions = &mut virtual_machine_data.instructions;

    let destination_register = instruction_decoder::decode_destination_register(instruction);

    let first_half = get_next_instruction(*registers, *instructions);
    let second_half = get_next_instruction(*registers, *instructions);
    let number = instruction_decoder::merge_u32s(first_half, second_half);
    let number = number as i64;
    load_i64_to_register(*registers, destination_register, number);
}

#[inline(always)]
pub fn load_nil_to_register(
    instruction: Instruction,
    virtual_machine_data: &mut VirtualMachineData,
) {
    let registers = &mut virtual_machine_data.registers;

    let destination = instruction_decoder::decode_destination_register(instruction);
    let register = Register::new(RegisterValueKind::None, 0);
    set_value_in_register(*registers, destination, register);
}

#[inline(always)]
pub fn load_bool_to_register(
    instruction: Instruction,
    virtual_machine_data: &mut VirtualMachineData,
) {
    let registers = &mut virtual_machine_data.registers;

    let destination = instruction_decoder::decode_destination_register(instruction);
    let boolean = instruction_decoder::decode_immutable_address_small(instruction);
    let register = Register::new(RegisterValueKind::Float64, boolean as u64);
    set_value_in_register(*registers, destination, register);
}

/// defines an empty global variable in the virtual machine
#[inline(always)]
pub fn define_global_indirect(
    instruction: Instruction,
    virtual_machine_data: &mut VirtualMachineData,
) {
    let immutables = &mut virtual_machine_data.immutables;
    let identifiers = &mut virtual_machine_data.identifiers;
    let globals = &mut virtual_machine_data.globals;

    let index = instruction_decoder::decode_immutable_address_small(instruction);
    let immutable = immutables[index as usize].clone();

    if let NovaObject::String(name) = immutable {
        let global_location = allocate_global(*globals);
        create_global(*identifiers, name.to_string(), global_location);
    }
}

/// store a value in a global location by first looking up name in the immutables array
#[inline(always)]
pub fn store_global_indirect(
    instruction: Instruction,
    virtual_machine_data: &mut VirtualMachineData,
) {
    let registers = &mut virtual_machine_data.registers;
    let immutables = &mut virtual_machine_data.immutables;
    let identifiers = &mut virtual_machine_data.identifiers;
    let globals = &mut virtual_machine_data.globals;
    let mem_cache = &mut virtual_machine_data.mem_cache;
    let memory = &mut virtual_machine_data.memory;

    let source = instruction_decoder::decode_source_register_1(instruction);
    let index = instruction_decoder::decode_immutable_address_small(instruction);

    let register = get_register(*registers, source);

    if let Some(&address) = mem_cache.get_cache(&(index as usize)) {
        set_global_value(*globals, address as u32, register);
        clear_register(*registers, source);
        return;
    }

    let immutable = unsafe { immutables.get_unchecked(index as usize) };

    if let NovaObject::String(name) = immutable {
        let global_address = identifiers.get(name.as_str());

        if let Some(&address) = global_address {
            mem_cache.add_cache(index as usize, address as usize);
            set_global_value(*globals, address, register);

            return;
        }

        emit_error_with_message(
            *registers,
            *memory,
            &format!("Cannot find global named: {}", name),
        );
        clear_register(*registers, source);
        return;
    }

    emit_error_with_message(
        *registers,
        *memory,
        &format!("Invalid global identifier: {:?}", immutable),
    );
    clear_register(*registers, source)
}

/// load a value from a global location into a register by first looking up its name in the immutables array
#[inline(always)]
pub fn load_global_indirect(
    instruction: Instruction,
    virtual_machine_data: &mut VirtualMachineData,
) {
    let registers = &mut virtual_machine_data.registers;
    let immutables = &mut virtual_machine_data.immutables;
    let identifiers = &mut virtual_machine_data.identifiers;
    let globals = &mut virtual_machine_data.globals;
    let mem_cache = &mut virtual_machine_data.mem_cache;
    let memory = &mut virtual_machine_data.memory;

    let destination = instruction_decoder::decode_destination_register(instruction);
    let index = instruction_decoder::decode_immutable_address_small(instruction);

    if let Some(&address) = mem_cache.get_cache(&(index as usize)) {
        load_global_value(*registers, *globals, destination, address as u32);
        return;
    }

    let immutable = unsafe { immutables.get_unchecked(index as usize) };

    if let NovaObject::String(name) = immutable {
        let global_address = identifiers.get(name.as_str());

        if let Some(&address) = global_address {
            mem_cache.add_cache(index as usize, address as usize);
            load_global_value(*registers, *globals, destination, address);

            return;
        }

        emit_error_with_message(
            *registers,
            *memory,
            &format!("Cannot find global named: {}", name),
        );
        return;
    }

    emit_error_with_message(
        *registers,
        *memory,
        &format!("Invalid global identifier: {:?}", immutable),
    );
}

#[inline(always)]
pub fn allocate_locals(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let locals = &mut virtual_machine_data.locals;
    // number of variables
    let number = instruction_decoder::decode_immutable_address_small(instruction);
    allocate_local_variables(*locals, number);
}

#[inline(always)]
pub fn deallocate_locals(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let locals = &mut virtual_machine_data.locals;
    // number of variables
    let number = instruction_decoder::decode_immutable_address_small(instruction) as usize;

    locals.drain(locals.len() - number..);
}

#[inline(always)]
pub fn store_local(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;
    let locals = &mut virtual_machine_data.locals;

    let source = instruction_decoder::decode_source_register_1(instruction);
    let address = instruction_decoder::decode_immutable_address_small(instruction);

    let register = get_register(*registers, source);
    let local_offset = unsafe { registers.get_unchecked(RegisterID::RLO as usize).value };
    unsafe {
        let local = locals.get_unchecked_mut((address + local_offset as u32) as usize);

        local.value = register.value;
        local.kind = register.kind;
    }

    clear_register(*registers, source);
}

#[inline(always)]
pub fn load_local(instruction: Instruction, virtual_machine_data: &mut VirtualMachineData) {
    let registers = &mut virtual_machine_data.registers;
    let locals = &mut virtual_machine_data.locals;

    let destination = instruction_decoder::decode_destination_register(instruction);
    let address = instruction_decoder::decode_immutable_address_small(instruction);

    let register = unsafe {
        let local_offset = get_register(*registers, RegisterID::RLO as u32).value;
        *locals.get_unchecked((address as u64 + local_offset) as usize)
    };

    set_value_in_register(*registers, destination, register);
}
