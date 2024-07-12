use nova::{
    instruction::InstructionBuilder, machine::VirtualMachine, object::NovaObject, program::Program,
};

fn main() {
    let mut vm = VirtualMachine::new();
    let program = get_program();
    vm.load_program(program);
    vm.start_vm(0);
}

fn get_program() -> Program {
    let immutables = vec![
        NovaObject::String(Box::new("number1".to_string())),
        NovaObject::String(Box::new("number2".to_string())),
    ];

    let instructions = vec![
        // define global variables
        InstructionBuilder::new_define_global_indirect(0u32),
        InstructionBuilder::new_define_global_indirect(1u32),
        // load constant numbers into register
        InstructionBuilder::new_load_float32_instruction(0),
        1000f32.to_bits(),
        InstructionBuilder::new_load_float32_instruction(1),
        88f32.to_bits(),
        // print register values
        InstructionBuilder::new_print_instruction(0, true),
        InstructionBuilder::new_print_instruction(1, true),
        // store the numbers into the named global variables
        InstructionBuilder::new_store_global_indirect(0, 0),
        InstructionBuilder::new_store_global_indirect(1, 1),
        // load the global variables into registers, switching the order
        InstructionBuilder::new_load_global_indirect(1, 0),
        InstructionBuilder::new_load_global_indirect(0, 1),
        // print the global variables
        InstructionBuilder::new_print_instruction(0, true),
        InstructionBuilder::new_print_instruction(1, true),
        InstructionBuilder::new_halt_instruction(),
    ];
    Program {
        instructions,
        immutables,
    }
}
