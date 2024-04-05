use nova::{compiler, machine::VirtualMachine};

fn main() {
    //let source = "1 + 5 * 7\n";
    let source = "a := 4\nb := a*a\nc := b * a\n d := 100\ne := a + b + c + d\n";
    let source2 = "a = e*b\nprintln(a)\n";
    let program = compiler::compile(source).unwrap();
    let program2 = compiler::compile(source2).unwrap();

    let mut vm = VirtualMachine::new();
    let mut offset = 0u32;
    vm.load_program(program);
    vm.start_vm(offset);

    offset += vm.get_instruction_count();
    vm.load_program(program2);
    vm.start_vm(offset)
}