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
    let immutables = vec![NovaObject::Number(1.0), NovaObject::Number(20.0)];

    let instructions = vec![
        InstructionBuilder::new_load_constant_instruction(0, 0),
        InstructionBuilder::new_load_constant_instruction(1, 1),
        InstructionBuilder::new_move_instruction(2, 0),
        InstructionBuilder::new_comparison_instruction(OpCode::LESSJ, 0, 1),
        InstructionBuilder::new_jump_instruction(3, true),
        InstructionBuilder::new_print_instruction(0, true),
        InstructionBuilder::new_binary_op_instruction(OpCode::Add, 0, 0, 2),
        InstructionBuilder::new_jump_instruction(5, false),
        InstructionBuilder::new_halt_instruction(),
    ];
    Program {
        instructions,
        immutables,
    }
}
