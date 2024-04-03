use nova::{compiler, machine::VirtualMachine};

fn main() {
    let source = "(9+3) / (2+1) * (5+5)\n";
    let program = compiler::compile(source).unwrap();

    let mut vm = VirtualMachine::new();
    vm.load_program(program);
    vm.start_vm(0);
}