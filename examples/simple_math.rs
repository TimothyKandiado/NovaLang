// Testing execution of bytecodes for simple mathematics
use nova::{
    bytecode::OpCode, instruction::InstructionBuilder, machine::VirtualMachine, object::NovaObject,
    program::Program,
};

fn main() {
    let mut vm = VirtualMachine::new();
    let program = get_program();
    vm.load_program(program);
    vm.start_vm(0);
}

fn get_program() -> Program {
    let immutables = vec![NovaObject::String(Box::new("I am Timothy".to_string()))];

    let instructions = vec![
        InstructionBuilder::new_load_float32_instruction(0),
        10.0f32.to_bits(),
        InstructionBuilder::new_load_float32_instruction(1),
        15.0f32.to_bits(),
        InstructionBuilder::new_binary_op_instruction(OpCode::Add, 0, 0, 1),
        InstructionBuilder::new_print_instruction(0, true),
        InstructionBuilder::new_binary_op_instruction(OpCode::Mod, 0, 0, 1),
        InstructionBuilder::new_print_instruction(0, true),
        InstructionBuilder::new_load_constant_instruction(2, 0),
        InstructionBuilder::new_print_instruction(2, true),
        InstructionBuilder::new_binary_op_instruction(OpCode::Add, 0, 0, 2),
        InstructionBuilder::new_print_instruction(0, true),
        InstructionBuilder::new_halt_instruction(),
    ];
    Program {
        instructions,
        immutables,
    }
}
