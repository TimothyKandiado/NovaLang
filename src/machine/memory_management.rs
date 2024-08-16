use crate::{
    instruction::Instruction,
    object::{MappedMemory, NovaObject},
    register::Register,
};

/// load an object from memory given the memory location
#[inline(always)]
pub fn load_object_from_memory(memory: &[NovaObject], address: u64) -> &NovaObject {
    unsafe {
        let object = memory.get_unchecked(address as usize);
        return object;
    }
    // &self.memory[address as usize]
}

/// store a NovaObject in the memory and return its allocated address
#[inline(always)]
pub fn store_object_in_memory(memory: &mut Vec<NovaObject>, object: NovaObject) -> Instruction {
    memory.push(object);
    let address = memory.len() - 1;
    address as Instruction
}

/// allocate memory on the globals vector
#[inline(always)]
pub fn allocate_global(globals: &mut Vec<Register>) -> Instruction {
    globals.push(Register::default());
    (globals.len() - 1) as Instruction
}

/// bind a global location to an identifier
#[inline(always)]
pub fn create_global(identifiers: &mut MappedMemory, name: String, global_location: Instruction) {
    identifiers.insert(name, global_location);
}

/// set value of a specified global location
#[inline(always)]
pub fn set_global_value(globals: &mut [Register], address: Instruction, new_value: Register) {
    unsafe {
        let global = globals.get_unchecked_mut(address as usize);
        global.kind = new_value.kind;
        global.value = new_value.value;
    }
}

/// load value from a specified global address
#[inline(always)]
pub fn load_global_value(
    registers: &mut [Register],
    globals: &mut [Register],
    destination: Instruction,
    global_address: Instruction,
) {
    let value = unsafe { *globals.get_unchecked(global_address as usize) };
    unsafe {
        let register = registers.get_unchecked_mut(destination as usize);

        register.kind = value.kind;
        register.value = value.value;
    }
}

#[inline(always)]
pub fn allocate_local_variables(locals: &mut Vec<Register>, number_of_locals: Instruction) {
    let mut local_space = vec![Register::default(); number_of_locals as usize];
    locals.append(&mut local_space)
}

#[inline(always)]
pub fn deallocate_local_variables(locals: &mut Vec<Register>, number_of_locals: Instruction) {
    let number = number_of_locals as usize;

    locals.drain(locals.len() - number..);
}
