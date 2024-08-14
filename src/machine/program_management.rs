use crate::{frame::Frame, instruction::Instruction, object::{NovaObject, RegisterValueKind}, register::{Register, RegisterID}};

use super::{array_copy, memory_management::{allocate_local_variables, deallocate_local_variables, store_object_in_memory}, register_management::{clear_registers, get_register, load_memory_address_to_register}};


#[inline(always)]
pub fn peek_next_instruction(registers: &[Register], instructions: &[Instruction]) -> Instruction {
    unsafe {
        let instruction_pointer = registers.get_unchecked(RegisterID::RPC as usize).value as usize;
        return *instructions.get_unchecked(instruction_pointer);
    }
}

#[inline(always)]
pub fn get_next_instruction(registers: &mut [Register], instructions: &[Instruction]) -> Instruction {
    let instruction = peek_next_instruction(registers, instructions);
    unsafe {
        registers.get_unchecked_mut(RegisterID::RPC as usize).value += 1;
    }
    
    instruction
}

#[inline(always)]
pub fn new_frame(registers: &mut [Register], frames: &mut Vec<Frame>, locals: &mut Vec<Register>, num_locals: Instruction) -> Frame {
    let return_address = unsafe {registers.get_unchecked(RegisterID::RPC as usize).value};
    let local_offset = unsafe {registers.get_unchecked(RegisterID::RLO as usize).value};

    let mut old_registers = [Register::empty(); RegisterID::RMax as usize + 1];
    old_registers.copy_from_slice(registers);

    let frame = Frame::new(old_registers, return_address, local_offset, false);

    frames.push(frame.clone());
    clear_registers(registers);

    set_local_offset(registers, locals);

    allocate_local_variables(locals, num_locals);

    unsafe {
        let register = registers.get_unchecked_mut(RegisterID::RMax as usize);
        register.kind = RegisterValueKind::MemAddress;
        register.value = num_locals as u64;
    }
    
    frame
}

#[inline(always)]
pub fn drop_frame(registers: &mut [Register], frames: &mut Vec<Frame>, locals: &mut Vec<Register>, running_state: &mut bool) {
    let return_value = unsafe {*registers.get_unchecked(RegisterID::RRTN as usize)};
    let num_locals = unsafe {registers.get_unchecked(RegisterID::RMax as usize).value};

    deallocate_local_variables(locals, num_locals as u32);

    let frame = frames.pop();

    if let Some(frame) = frame {
        if frame.is_main {
            *running_state = false;
            return;
        }

        array_copy(&frame.registers, 0, registers, 0, registers.len());

        unsafe {
            
            let register = registers.get_unchecked_mut(RegisterID::RRTN as usize);
            register.kind = return_value.kind;
            register.value = return_value.value;
        }
        
    } else {
        *running_state = false;
    }
}

#[inline(always)]
pub fn set_local_offset(registers: &mut [Register], locals: &[Register]) {
    unsafe {
        registers.get_unchecked_mut(RegisterID::RLO as usize).value = (locals.len()) as u64;
    }
}

#[inline(always)]
pub fn emit_error_with_message(registers: &mut [Register], memory: &mut Vec<NovaObject>, message: &str) {
    let address = store_object_in_memory(memory, NovaObject::String(Box::new(message.to_string())));
    load_memory_address_to_register(registers, RegisterID::RERR as Instruction, address);
}

#[inline(always)]
pub fn print_error(registers: &[Register], memory: &[NovaObject]) {
    let register = get_register(registers, RegisterID::RERR as Instruction);

    if let RegisterValueKind::MemAddress = register.kind {
        let address = register.value;
        let object = &memory[address as usize];
        eprint!("Error: ");

        if let NovaObject::String(string) = object {
            eprint!("{}", string)
        }
        eprintln!();
    }
}
    
#[inline(always)]
pub fn check_error(registers: &[Register]) -> bool {
    let register = get_register(registers, RegisterID::RERR as Instruction);

    if let RegisterValueKind::MemAddress = register.kind {
        return true;
    }

    false
}