use nova::{compiler, machine::VirtualMachine};

fn main() {
    //let source = "1 + 5 * 7\n";
    test_block_locals();
}

fn test_block_locals() {
    let source = "block\na := 10\nb := 20\nprintln(a*b)\nend\n";
    let program = compiler::compile(source).unwrap();

    let mut vm = VirtualMachine::new();
    let offset = 0u32;
    vm.load_program(program);
    let code = vm.start_vm(offset);
    assert_eq!(code, 0);
}