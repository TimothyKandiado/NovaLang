use nova::{
    bytecode::OpCode, instruction::InstructionBuilder, machine::VirtualMachine, program::Program,
};

fn main() {
    let mut vm = VirtualMachine::new();
    let program = get_program();
    vm.load_program(program);
    vm.start_vm(0);
}

fn get_program() -> Program {
    let immutables = Vec::new();
    let instructions = vec![
        InstructionBuilder::new_allocate_local(2),
        InstructionBuilder::new_load_float32_instruction(0),
        100f32.to_bits(),
        InstructionBuilder::new_load_float32_instruction(1),
        (-60f32).to_bits(),
        InstructionBuilder::new_store_local(0, 0),
        InstructionBuilder::new_store_local(1, 1),
        InstructionBuilder::new_load_local(0, 0),
        InstructionBuilder::new_load_local(1, 1),
        InstructionBuilder::new_print_instruction(0, true),
        InstructionBuilder::new_print_instruction(1, true),
        // New call frame
        InstructionBuilder::new()
            .add_opcode(OpCode::NewFrame)
            .build(),
        InstructionBuilder::new_jump_instruction(2, true),
        InstructionBuilder::new_jump_instruction(14, true),
        InstructionBuilder::new_allocate_local(2),
        InstructionBuilder::new_load_float32_instruction(0),
        777f32.to_bits(),
        InstructionBuilder::new_load_float32_instruction(1),
        (-987f32).to_bits(),
        InstructionBuilder::new_store_local(0, 0),
        InstructionBuilder::new_store_local(1, 1),
        InstructionBuilder::new_load_local(0, 0),
        InstructionBuilder::new_load_local(1, 1),
        InstructionBuilder::new_print_instruction(0, true),
        InstructionBuilder::new_print_instruction(1, true),
        // End Frame
        InstructionBuilder::new_deallocate_local(2),
        InstructionBuilder::new()
            .add_opcode(OpCode::ReturnNone)
            .build(),
        InstructionBuilder::new_load_local(0, 0),
        InstructionBuilder::new_load_local(1, 1),
        InstructionBuilder::new_print_instruction(0, true),
        InstructionBuilder::new_print_instruction(1, true),
        InstructionBuilder::new_deallocate_local(2),
        InstructionBuilder::new_halt_instruction(),
    ];

    Program {
        instructions,
        immutables,
    }
}
