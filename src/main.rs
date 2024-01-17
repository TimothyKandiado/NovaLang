use nova::{machine::VirtualMachine, program::Program, object::NovaObject, instruction::InstructionBuilder, bytecode::OpCode};

fn main() {
    let mut vm = VirtualMachine::new();
    let program = get_program();
    vm.load_program(program);
    vm.start_vm(0);

    #[cfg(feature = "debug")]
    print_mem_usage(&vm);
}

#[cfg(feature = "debug")]
fn print_mem_usage(vm: &VirtualMachine) {
    use std::mem::size_of_val;

    let stack = size_of_val(vm);
    println!("vm stack usage = {} bytes", stack)
}

fn get_program() -> Program {
    let immutables = vec![
        NovaObject::Number(10.0), 
        NovaObject::Number(15.0), 
        NovaObject::String(Box::new("I am Timothy".to_string()))
        ];
    
    let instructions = vec![
        InstructionBuilder::new_load_constant_instruction(0, 0),
        InstructionBuilder::new_load_constant_instruction(1, 1),
        InstructionBuilder::new_binary_op_instruction(OpCode::Add, 0, 0, 1),
        InstructionBuilder::new_print_instruction(0, true),
        InstructionBuilder::new_binary_op_instruction(OpCode::Mod, 0, 0, 1),
        InstructionBuilder::new_print_instruction(0, true),
        InstructionBuilder::new_load_constant_instruction(2, 2),
        InstructionBuilder::new_print_instruction(2, true),
        InstructionBuilder::new_binary_op_instruction(OpCode::Add, 0, 0, 2),
        InstructionBuilder::new_print_instruction(0, true),
        InstructionBuilder::new_halt_instruction(),
    ];
    Program {instructions, immutables}
}
