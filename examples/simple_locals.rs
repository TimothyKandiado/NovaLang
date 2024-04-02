use nova::{
    instruction::InstructionBuilder, machine::VirtualMachine, program::Program
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

        InstructionBuilder::new_print_instruction(0, true),
        InstructionBuilder::new_print_instruction(1, true),

        InstructionBuilder::new_store_local(0, 0),
        InstructionBuilder::new_store_local(1, 1),

        InstructionBuilder::new_load_local(1, 0),
        InstructionBuilder::new_load_local(0, 1),

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
