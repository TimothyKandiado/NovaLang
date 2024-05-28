#[cfg(test)]
mod tests{
    use crate::{compiler, machine::VirtualMachine};

    #[test]
    fn test_globals() {
        let source = "a := 4\nb := a*a\nc := b * a\n d := 100\ne := a + b + c + d\n";
        let source2 = "a = e*b\nprintln(a)\n";
        let program = compiler::compile(source).unwrap();
        let program2 = compiler::compile(source2).unwrap();

        let mut vm = VirtualMachine::new();
        let mut offset = 0u32;
        vm.load_program(program);
        let code = vm.start_vm(offset);
        assert_eq!(code, 0);

        offset += vm.get_instruction_count();
        vm.load_program(program2);
        let code = vm.start_vm(offset);
        assert_eq!(code, 0);
    }

    #[test]
    fn test_block_locals() {
        let source = "block\na := 10\nb := 20\nprintln(a*b)\nend\n";
        let program = compiler::compile(source).unwrap();

        let mut vm = VirtualMachine::new();
        let offset = 0u32;
        vm.load_program(program);
        let code = vm.start_vm(offset);
        assert_eq!(code, 0);
    }
}