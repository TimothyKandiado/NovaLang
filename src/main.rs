use std::mem::size_of_val;

use nova::{machine::VirtualMachine, program::Program, object::NovaObject, instruction::InstructionBuilder, bytecode::OpCode, register::RegisterID};

fn main() {
    let mut vm = VirtualMachine::new();
    let program = get_program();
    vm.load_program(program);
    vm.start_vm(0);
    println!("size of vm = {} bytes", size_of_val(&vm))
}

fn get_program() -> Program {
    let immutables = vec![NovaObject::Number(10.0), NovaObject::Number(15.0)];
    let instruction1 = InstructionBuilder::new()
        .add_opcode(OpCode::LoadK)
        .add_destination_register(RegisterID::R0 as u32)
        .add_immutable_address_small(0)
        .build();

    let instruction2 = InstructionBuilder::new()
        .add_opcode(OpCode::LoadK)
        .add_destination_register(RegisterID::R1 as u32)
        .add_immutable_address_small(1)
        .build();

    let instruction3 = InstructionBuilder::new()
        .add_opcode(OpCode::Add)
        .add_destination_register(RegisterID::R0 as u32)
        .add_source_register_1(RegisterID::R0 as u32)
        .add_source_register_2(RegisterID::R1 as u32)
        .build();

    let instruction4 = InstructionBuilder::new()
        .add_opcode(OpCode::Halt)
        .build();

    let instructions = vec![instruction1, instruction2, instruction3, instruction4];
    Program {instructions, immutables}
}
