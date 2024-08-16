pub mod bytecode_execution;
pub mod memory_management;
pub mod program_management;
pub mod register_management;

use std::ptr::copy_nonoverlapping;

use memory_management::{allocate_global, create_global, set_global_value, store_object_in_memory};
use program_management::{check_error, emit_error_with_message, get_next_instruction};
use register_management::get_register;

use crate::{
    bytecode::{OpCode, BYTECODE_LOOKUP_TABLE},
    cache::MemoryCache,
    frame::Frame,
    instruction::{instruction_decoder, Instruction, InstructionBuilder},
    object::{
        MappedMemory, NativeFunction, NovaCallable, NovaFunctionID, NovaObject, RegisterValueKind,
    },
    program::{LineDefinition, Program},
    register::{Register, RegisterID},
};

#[cfg(feature = "debug")]
use crate::debug::debug_instruction;

const PC_START: Instruction = 0x0;

pub struct VirtualMachineData<'a> {
    pub instructions: &'a mut Vec<Instruction>,
    pub immutables: &'a mut Vec<NovaObject>,
    pub registers: &'a mut [Register; RegisterID::RMax as usize + 1],
    pub running: &'a mut bool,
    pub memory: &'a mut Vec<NovaObject>,
    pub frames: &'a mut Vec<Frame>,
    pub locals: &'a mut Vec<Register>,
    pub globals: &'a mut Vec<Register>,
    pub identifiers: &'a mut MappedMemory,
    pub mem_cache: &'a mut MemoryCache,
}

#[inline(always)]
fn offset_immutable_address(instruction: Instruction, offset: Instruction) -> Instruction {
    let opcode = instruction_decoder::decode_opcode(instruction);

    if opcode == OpCode::LoadK.to_u32()
        || opcode == OpCode::DefineGlobalIndirect.to_u32()
        || opcode == OpCode::LoadGlobalIndirect.to_u32()
        || opcode == OpCode::StoreGlobalIndirect.to_u32()
    {
        let old_address = instruction_decoder::decode_immutable_address_small(instruction);
        let new_address = old_address + offset;
        return InstructionBuilder::from(instruction)
            .clear_address_small()
            .add_address_small(new_address)
            .build();
    }

    instruction
}

pub struct VirtualMachine {
    instructions: Vec<Instruction>,
    immutables: Vec<NovaObject>,
    registers: [Register; RegisterID::RMax as usize + 1],
    running: bool,
    memory: Vec<NovaObject>,
    frames: Vec<Frame>,
    locals: Vec<Register>,
    globals: Vec<Register>,
    identifiers: MappedMemory,
    mem_cache: MemoryCache,
    line_definitions: Vec<LineDefinition>,
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            immutables: Vec::new(),
            registers: [Register::default(); RegisterID::RMax as usize + 1],
            running: false,
            memory: Vec::new(),
            frames: vec![Frame::main()],
            locals: Vec::new(),
            globals: Vec::new(),
            identifiers: MappedMemory::default(),
            mem_cache: MemoryCache::default(),
            line_definitions: Vec::new(),
        }
    }

    pub fn load_natives(&mut self, native_functions: Vec<NativeFunction>) {
        for native_function in native_functions {
            self.load_callable(NovaCallable::NativeFunction(&native_function));
        }
    }

    /// load a native function into virtual machine
    /// the function name is stored in the identifiers map together with an address to a global location.
    /// the global location points to a memory address containing a NovaObject wrapping the NativeFunction
    #[inline(always)]
    fn load_callable(&mut self, callable: NovaCallable) {
        let global_location = allocate_global(&mut self.globals);
        let name = callable.get_name().to_string();
        create_global(&mut self.identifiers, name, global_location);
        let nova_object = callable.as_object();
        let memory_location = store_object_in_memory(&mut self.memory, nova_object);

        let global_value = match callable {
            NovaCallable::NovaFunction(nova_function) => {
                let name_address = self.immutables.len() as u32 - 1;
                let nova_function_id =
                    NovaFunctionID::from_nova_function(nova_function, name_address);

                if let Some(nova_function_id) = nova_function_id {
                    let function_address = nova_function.address as u64;
                    Register::new(
                        RegisterValueKind::NovaFunctionID(nova_function_id),
                        function_address,
                    )
                } else {
                    Register::new(RegisterValueKind::MemAddress, memory_location as u64)
                }
            }

            _ => Register::new(RegisterValueKind::MemAddress, memory_location as u64),
        };

        set_global_value(&mut self.globals, global_location, global_value);
    }

    pub fn get_instruction_count(&self) -> u32 {
        self.instructions.len() as u32
    }

    pub fn load_program(&mut self, program: Program) {
        let immutable_offset = self.immutables.len() as Instruction;
        let instruction_offset = self.instructions.len();

        for &instruction in &program.instructions {
            //self.instruction_count += 1;
            // TODO: check validity of opcodes;
            let instruction = offset_immutable_address(instruction, immutable_offset);
            self.instructions.push(instruction);
        }

        for immutable in &program.immutables {
            if immutable.is_callable() {
                let callable = immutable.as_callable();
                self.load_callable(callable);
            }

            self.immutables.push(immutable.clone());
        }

        for line_definition in &program.line_definitions {
            let mut line_definition = line_definition.clone();
            line_definition.last_instruction += instruction_offset;
            self.line_definitions.push(line_definition)
        }
    }

    #[inline(always)]
    fn clear_error(&mut self) {
        self.registers[RegisterID::RERR as usize] = Register::empty();
    }

    #[inline(always)]
    fn print_error(&self) {
        let register = get_register(&self.registers, RegisterID::RERR as Instruction);

        if let RegisterValueKind::MemAddress = register.kind {
            let address = register.value;
            let object = &self.memory[address as usize];
            eprint!("Error: ");

            if let NovaObject::String(string) = object {
                eprint!("'{}'", string)
            }
            eprintln!(" Most recent call first");
        }

        let program_counter = self.registers[RegisterID::RPC as usize].value as usize;

        let line_definition = self.get_source_line_definition(program_counter);

        if let Some(line_definition) = line_definition {
            eprintln!(
                "On line [{}] in file '{}'",
                line_definition.source_line, line_definition.source_file
            );
        }

        if self.frames.len() <= 1 {
            return;
        }

        let frames = self.frames.iter().as_slice()[1..].iter().rev();

        for frame in frames {
            let program_counter = frame.return_address as usize - 2; // I don't understand why 2, but it works

            let line_definition = self.get_source_line_definition(program_counter);

            if let Some(line_definition) = line_definition {
                eprintln!(
                    "Called from line [{}] in file '{}'",
                    line_definition.source_line, line_definition.source_file
                );
            }
        }
    }

    fn get_source_line_definition(&self, program_counter: usize) -> Option<&LineDefinition> {
        let mut maximum_line_definition = self.line_definitions.get(0);

        for line_definition in self.line_definitions.iter() {
            if line_definition.last_instruction <= program_counter {
                maximum_line_definition = Some(line_definition);
            }
        }

        return maximum_line_definition;
    }

    pub fn start_vm(&mut self, offset: Instruction) -> u32 {
        self.running = true;
        let program_counter = Register {
            kind: RegisterValueKind::MemAddress,
            value: (offset + PC_START) as u64,
        };

        self.registers[RegisterID::RPC as usize] = program_counter;

        let mut virtual_machine_data = VirtualMachineData {
            registers: &mut self.registers,
            instructions: &mut self.instructions,
            immutables: &mut self.immutables,
            running: &mut self.running,
            memory: &mut self.memory,
            frames: &mut self.frames,
            locals: &mut self.locals,
            globals: &mut self.globals,
            identifiers: &mut self.identifiers,
            mem_cache: &mut self.mem_cache,
        };

        while *virtual_machine_data.running {
            #[cfg(feature = "debug")]
            debug(&virtual_machine_data);

            let instruction = get_next_instruction(
                virtual_machine_data.registers,
                virtual_machine_data.instructions,
            );

            Self::execute_instruction(instruction, &mut virtual_machine_data);

            if check_error(virtual_machine_data.registers) {
                self.print_error();
                self.clear_error();
                return 1;
            }
        }

        0
    }

    #[inline(always)]
    fn execute_instruction(
        instruction: Instruction,
        virtual_machine_data: &mut VirtualMachineData,
    ) {
        let opcode = instruction_decoder::decode_opcode(instruction);

        let opcode = unsafe { *BYTECODE_LOOKUP_TABLE.get_unchecked(opcode as usize) };

        match opcode {
            OpCode::NoInstruction => {}
            // System Interrupt
            OpCode::Halt => {
                *virtual_machine_data.running = false;
            }

            // Unary operations
            OpCode::Neg => {
                bytecode_execution::negate(instruction, virtual_machine_data);
            }

            // Binary Operations
            OpCode::Add => {
                bytecode_execution::add(instruction, virtual_machine_data);
            }
            OpCode::Sub => {
                bytecode_execution::sub(instruction, virtual_machine_data);
            }
            OpCode::Mul => {
                bytecode_execution::mul(instruction, virtual_machine_data);
            }
            OpCode::Div => {
                bytecode_execution::div(instruction, virtual_machine_data);
            }
            OpCode::Pow => {
                bytecode_execution::pow(instruction, virtual_machine_data);
            }
            OpCode::Mod => {
                bytecode_execution::modulus(instruction, virtual_machine_data);
            }

            // Register Manipulation
            OpCode::LoadK => {
                bytecode_execution::load_constant_to_register(instruction, virtual_machine_data);
            }

            OpCode::LoadNil => {
                bytecode_execution::load_nil_to_register(instruction, virtual_machine_data);
            }

            OpCode::LoadBool => {
                bytecode_execution::load_bool_to_register(instruction, virtual_machine_data);
            }

            OpCode::LoadFloat32 => {
                bytecode_execution::load_float32_to_register(instruction, virtual_machine_data);
            }

            OpCode::LoadFloat64 => {
                bytecode_execution::load_float64_to_register(instruction, virtual_machine_data);
            }

            OpCode::LoadInt32 => {
                bytecode_execution::load_int32_to_register(instruction, virtual_machine_data);
            }

            OpCode::LoadInt64 => {
                bytecode_execution::load_int64_to_register(instruction, virtual_machine_data);
            }

            OpCode::Move => {
                register_management::move_register(virtual_machine_data.registers, instruction);
            }

            // Variable Manipulation
            OpCode::DefineGlobalIndirect => {
                bytecode_execution::define_global_indirect(instruction, virtual_machine_data);
            }

            OpCode::StoreGlobalIndirect => {
                bytecode_execution::store_global_indirect(instruction, virtual_machine_data);
            }

            OpCode::LoadGlobalIndirect => {
                bytecode_execution::load_global_indirect(instruction, virtual_machine_data);
            }

            OpCode::LoadGlobal => {
                let destination = instruction_decoder::decode_destination_register(instruction);
                let address = instruction_decoder::decode_immutable_address_small(instruction);

                memory_management::load_global_value(
                    virtual_machine_data.registers,
                    virtual_machine_data.globals,
                    destination,
                    address,
                );
            }

            OpCode::AllocateLocal => {
                bytecode_execution::allocate_locals(instruction, virtual_machine_data);
            }

            OpCode::DeallocateLocal => {
                bytecode_execution::deallocate_locals(instruction, virtual_machine_data);
            }

            OpCode::StoreLocal => {
                bytecode_execution::store_local(instruction, virtual_machine_data);
            }

            OpCode::LoadLocal => {
                bytecode_execution::load_local(instruction, virtual_machine_data);
            }

            // Logical tests
            OpCode::Less => {
                bytecode_execution::less(instruction, virtual_machine_data);
            }

            OpCode::LessEqual => {
                bytecode_execution::less_or_equal(instruction, virtual_machine_data);
            }

            OpCode::Not => {
                bytecode_execution::not(instruction, virtual_machine_data);
            }

            OpCode::Equal => {
                bytecode_execution::equal(instruction, virtual_machine_data);
            }

            // Control flow
            OpCode::JumpFalse => {
                bytecode_execution::jump_if_false(instruction, virtual_machine_data);
            }

            OpCode::Jump => {
                bytecode_execution::jump(instruction, virtual_machine_data);
            }

            OpCode::Invoke => {
                bytecode_execution::invoke(instruction, virtual_machine_data);
            }

            OpCode::ReturnNone => {
                bytecode_execution::return_none(instruction, virtual_machine_data)
            }

            OpCode::ReturnVal => bytecode_execution::return_val(instruction, virtual_machine_data),

            OpCode::LoadReturn => {
                bytecode_execution::load_return(instruction, virtual_machine_data)
            }

            // IO
            OpCode::Print => bytecode_execution::print(instruction, virtual_machine_data),

            _ => emit_error_with_message(
                virtual_machine_data.registers,
                virtual_machine_data.memory,
                &format!("Unsupported opcode instruction ({:?})", opcode),
            ),
        }
    }
}

#[allow(dead_code)]
fn print_vec_of_registers(registers: &[Register]) {
    println!("[");
    for (index, register) in registers.iter().enumerate() {
        println!("\t[{}] {}, ", index, register)
    }
    println!("]");
}

#[inline(always)]
fn array_copy<T>(
    source: &[T],
    source_index: usize,
    destination: &mut [T],
    destination_index: usize,
    length: usize,
) {
    unsafe {
        let src = source.as_ptr().offset(source_index as isize);
        let dest = destination.as_mut_ptr().offset(destination_index as isize);

        copy_nonoverlapping(src, dest, length);
    }
}

#[cfg(feature = "debug")]
pub fn debug(vm: &VirtualMachineData) {
    #[cfg(feature = "dbg_code")]
    {
        let current = vm.registers[RegisterID::RPC as usize].value;
        println!(
            "[{}]: {}",
            current,
            debug_instruction(
                &vm.instructions,
                vm.registers[RegisterID::RPC as usize].value,
            )
        );
    }
    #[cfg(feature = "verbose")]
    print_register_values(vm);
    #[cfg(feature = "dbg_global")]
    print_globals(vm);
    #[cfg(feature = "dbg_local")]
    print_locals(vm);
    #[cfg(feature = "dbg_global")]
    print_identifiers(vm);
    #[cfg(feature = "dbg_memory")]
    print_memory(vm);
    #[cfg(feature = "dbg_step")]
    wait_for_input();
}

#[cfg(feature = "dbg_step")]
fn wait_for_input() {
    use std::io;

    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer);
}

#[cfg(feature = "verbose")]
fn print_register_values(vm: &VirtualMachineData) {
    println!("{:=^30}", "Registers");
    for register_index in 0..RegisterID::RMax as usize + 1 {
        let register = vm.registers[register_index];
        println!("==> R{:<2}: {}", register_index, register);
    }
    println!("{:=^30}", "");
}

#[cfg(feature = "dbg_memory")]
fn print_memory(vm: &VirtualMachineData) {
    println!("{:=^30}", "Heap");
    println!("==> {:?}", &vm.memory);
    println!("{:=^30}", "");
}

#[cfg(feature = "dbg_global")]
fn print_globals(vm: &VirtualMachineData) {
    println!("{:=^30}", "Globals");
    print_vec_of_registers(&vm.globals);
    println!("{:=^30}", "");
}

#[cfg(feature = "dbg_local")]
fn print_locals(vm: &VirtualMachineData) {
    println!("{:=^30}", "Locals");
    print_vec_of_registers(&vm.locals);
    println!("{:=^30}", "");
}

#[cfg(feature = "dbg_global")]
fn print_identifiers(vm: &VirtualMachineData) {
    println!("{:=^30}", "Identifiers");
    println!("==> {:?}", &vm.identifiers);
    println!("{:=^30}", "");
}

#[cfg(test)]
mod tests {
    use crate::machine::array_copy;

    #[test]
    fn test_array_copy() {
        let src = [1, 2, 3, 4, 5, 6];
        let mut dest = [0; 6];

        //println!("Source: {:?}", src);
        //println!("Destination before = {:?}", dest);
        array_copy(&src, 2, &mut dest, 0, 3);
        //println!("Destination after = {:?}", dest);
        let expected_dest = [3, 4, 5, 0, 0, 0];
        assert_eq!(expected_dest, dest);
    }
}
