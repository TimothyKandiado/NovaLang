use rustc_hash::FxHashSet;

use crate::{object::NovaObject, register::{Register, RegisterID}};

use super::VirtualMachineData;

/// traverses through registers, local variables, and global variables checking if they contain a reference to memory (heap)
/// and marks those objects as live
#[inline(always)]
pub fn mark_all_live_objects(vm_data: &mut VirtualMachineData) -> FxHashSet<usize> {
    let mut live_object_set = FxHashSet::default();

    mark_live_objects_from_registers(&vm_data.registers, &mut live_object_set);
    mark_live_objects_from_variables(&vm_data.locals, &mut live_object_set);
    mark_live_objects_from_variables(&vm_data.globals, &mut live_object_set);
    
    live_object_set
}

/// sets all memory locations not present in live_objects_set as NovaObject::None (freeing the memory)
#[inline(always)]
pub fn clean_all_dead_objects(memory: &mut [NovaObject], live_objects: &FxHashSet<usize>) -> Vec<usize> {
    let mut freed_memory = Vec::new();
    if live_objects.len() == memory.len() {
        return freed_memory;
    }
    memory.iter_mut().enumerate().for_each(|(address, object)| {
        if !live_objects.contains(&address) {
            *object = NovaObject::None;
            freed_memory.push(address);
        }
    });

    freed_memory
}

/// retrieve of memory locations referenced in the registers 
/// and return them as live_objects
#[inline(always)]
fn mark_live_objects_from_registers(registers: &[Register; RegisterID::RMax as usize + 1], live_object_set: &mut FxHashSet<usize>) {
    registers[0..=RegisterID::R15 as usize].iter().for_each(|register| {
        if register.kind.is_mem_address() {
            live_object_set.insert(register.value as usize);
        }
    })
}

/// retrieve of memory locations referenced in the registers 
/// and return them as live_objects
#[inline(always)]
fn mark_live_objects_from_variables(variables: &[Register], live_object_set: &mut FxHashSet<usize>) {
    variables.iter().for_each(|register| {
        if register.kind.is_mem_address() {
            live_object_set.insert(register.value as usize);
        }
    })
}

