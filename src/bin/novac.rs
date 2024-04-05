use nova::{compiler, machine::VirtualMachine};

fn main() {
    let source = "a := 4\nb := a*a\nc := b * a\n d := 100\ne := a + b + c + d\n";
    let program = compiler::compile(source).unwrap();

    let mut vm = VirtualMachine::new();
    vm.load_program(program);
    vm.start_vm(0);
}