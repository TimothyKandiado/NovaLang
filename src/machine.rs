use crate::{
    bytecode::{OpCode,BYTECODE_LOOKUP_TABLE},
    frame::Frame,
    instruction::{instruction_decoder, Instruction, InstructionBuilder},
    object::{MappedMemory, NativeFunction, NovaCallable, NovaObject, RegisterValueKind},
    program::Program,
    register::{Register, RegisterID},
};

#[cfg(feature = "debug")]
use crate::debug::debug_instruction;

const PC_START: Instruction = 0x0;

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
        let global_location = self.allocate_global();
        let name = callable.get_name().to_string();
        self.create_global(name, global_location);
        let nova_object = callable.as_object();
        let memory_location = self.store_object_in_memory(nova_object);
        let global_value = Register::new(RegisterValueKind::MemAddress, memory_location as u64);
        self.set_global_value(global_location, global_value);
    }

    pub fn get_instruction_count(&self) -> u32 {
        self.instructions.len() as u32
    }

    pub fn load_program(&mut self, program: Program) {
        let immutable_offset = self.immutables.len() as Instruction;

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
    }

    #[inline(always)]
    fn clear_error(&mut self) {
        self.registers[RegisterID::RERR as usize] = Register::empty();
    }

    pub fn start_vm(&mut self, offset: Instruction) -> u32 {
        self.running = true;
        let program_counter = Register {
            kind: RegisterValueKind::MemAddress,
            value: (offset + PC_START) as u64,
        };

        self.registers[RegisterID::RPC as usize] = program_counter;

        while self.running {
            #[cfg(feature = "debug")]
            self.debug();

            let instruction = self.get_next_instruction();

            self.execute_instruction(instruction);

            if self.check_error() {
                self.print_error();
                self.clear_error();
                return 1;
            }
        }
        
        0
    }

    #[inline(always)]
    fn execute_instruction(&mut self, instruction: Instruction) {
        let opcode = instruction_decoder::decode_opcode(instruction);

        let opcode = unsafe {
            *BYTECODE_LOOKUP_TABLE.get_unchecked(opcode as usize)
        };

        match opcode {
            OpCode::NoInstruction => {
            }
            // System Interrupt
            OpCode::Halt => {
                self.running = false;
            }

            // Unary operations
            OpCode::Neg => {
                self.negate(instruction);
            }

            // Binary Operations
            OpCode::Add => {
                self.add(instruction);
            }
            OpCode::Sub => {
                self.sub(instruction);
            }
            OpCode::Mul => {
                self.mul(instruction);
            }
            OpCode::Div => {
                self.div(instruction);
            }
            OpCode::Pow => {
                self.pow(instruction);
            }
            OpCode::Mod => {
                self.modulus(instruction);
            }

            // Register Manipulation
            OpCode::LoadK => {
                self.load_constant_to_register(instruction);
            }

            OpCode::LoadNil => {
                self.load_nil_to_register(instruction);
            }

            OpCode::LoadBool => {
                self.load_bool_to_register(instruction);
            }

            OpCode::LoadFloat32 => {
                self.load_float32_to_register(instruction);
            }

            OpCode::LoadFloat64 => {
                self.load_float64_to_register(instruction);
            }

            OpCode::LoadInt32 => {
                self.load_int32_to_register(instruction);
            }

            OpCode::LoadInt64 => {
                self.load_int64_to_register(instruction);
            }

            OpCode::Move => {
                self.move_register(instruction);
            }

            // Variable Manipulation
            OpCode::DefineGlobalIndirect => {
                self.define_global_indirect(instruction);
            }

            OpCode::StoreGlobalIndirect => {
                self.store_global_indirect(instruction);
            }

            OpCode::LoadGlobalIndirect => {
                self.load_global_indirect(instruction);
            }

            OpCode::LoadGlobal => {
                let destination = instruction_decoder::decode_destination_register(instruction);
                let address = instruction_decoder::decode_immutable_address_small(instruction);

                self.load_global_value(destination, address);
            }

            OpCode::AllocateLocal => {
                self.allocate_locals(instruction);
            }

            OpCode::DeallocateLocal => {
                self.deallocate_locals(instruction);
            }

            OpCode::StoreLocal => {
                self.store_local(instruction);
            }

            OpCode::LoadLocal => {
                self.load_local(instruction);
            }

            // Logical tests
            OpCode::Less => {
                self.less(instruction);
            }

            OpCode::LessEqual => {
                self.less_or_equal(instruction);
            }

            OpCode::Not => {
                self.not(instruction);
            }

            OpCode::Equal => {
                self.equal(instruction);
            }

            // Control flow
            OpCode::JumpFalse => {
                self.jump_if_false(instruction);
            }

            OpCode::Jump => {
                self.jump(instruction);
            }

            OpCode::Invoke => {
                self.invoke(instruction);
            }

            OpCode::ReturnNone => self.return_none(),

            OpCode::ReturnVal => self.return_val(instruction),

            OpCode::LoadReturn => self.load_return(instruction),

            // IO
            OpCode::Print => {
                self.print(instruction);
            }

            _ => self.emit_error_with_message(&format!(
                "Unsupported opcode instruction ({:?})",
                opcode
            )),
        }
    }
    #[inline(always)]
    fn move_register(&mut self, instruction: Instruction) {
        let destination = instruction_decoder::decode_destination_register(instruction);
        let source = instruction_decoder::decode_source_register_1(instruction);

        let value = self.get_register(source);
        self.set_value_in_register(destination, value);
    }

    /// Clear the temporary value registers
    #[inline(always)]
    fn clear_registers(&mut self) {
        for index in 1..RegisterID::R9 as usize {
            self.registers[index] = Register::empty();
        }
    }

    #[inline(always)]
    fn new_frame(&mut self, num_locals: Instruction) -> Frame {
        let return_address = self.registers[RegisterID::RPC as usize].value;
        let local_offset = self.registers[RegisterID::RLO as usize].value;

        let old_registers = self.registers;
        let frame = Frame::new(old_registers, return_address, local_offset, false);

        self.frames.push(frame.clone());
        self.clear_registers();

        self.set_local_offset();

        self.allocate_local_variables(num_locals);
        self.registers[RegisterID::RMax as usize] = Register::new(RegisterValueKind::MemAddress, num_locals as u64);

        frame
    }

    #[inline(always)]
    fn set_local_offset(&mut self) {
        self.registers[RegisterID::RLO as usize].value = (self.locals.len()) as u64;
    }

    #[inline(always)]
    fn drop_frame(&mut self) {
        let return_value = self.registers[RegisterID::RRTN as usize];
        let num_locals = self.registers[RegisterID::RMax as usize].value;
        self.deallocate_local_variables(num_locals as u32);

        let frame = self.frames.pop();

        if let Some(frame) = frame {
            if frame.is_main {
                self.running = false;
                return;
            }

            self.registers = frame.registers;
            self.registers[RegisterID::RRTN as usize] = return_value;
        } else {
            self.running = false;
        }
    }

    #[inline(always)]
    fn invoke(&mut self, instruction: Instruction) {
        let invoke_register = instruction_decoder::decode_source_register_2(instruction);
        let argument_start = instruction_decoder::decode_destination_register(instruction);
        let argument_number = instruction_decoder::decode_source_register_1(instruction);

        let register = self.get_register(invoke_register);

        if register.kind != RegisterValueKind::MemAddress {
            self.emit_error_with_message("Function not found");
            return
        }

        let nova_object = self.load_object_from_memory(register.value);

        let callable = match nova_object {
            NovaObject::NovaFunction(nova_function) => NovaCallable::NovaFunction(nova_function),
            NovaObject::NativeFunction(native_function) => NovaCallable::NativeFunction(native_function),
            _ => NovaCallable::None
        };

        match callable {
            NovaCallable::NovaFunction(function) => {
                let address = function.address;

                if argument_number != function.arity {
                    self.emit_error_with_message(&format!(
                        "Not enough function arguments.\n{} are required\n{} were provided",
                        function.arity, argument_number
                    ));
                    return;
                }

                let num_locals = function.number_of_locals;
                let old_frame = self.new_frame(num_locals);

                let mut source_index = argument_start as usize;
                let source_end = (argument_start + argument_number) as usize;
                let mut destination_index = 0;

                while source_index < source_end {
                    self.registers[destination_index] = old_frame.registers[source_index];
                    destination_index += 1;
                    source_index += 1;
                }

                self.registers[RegisterID::RPC as usize].value = address as u64;
            }

            NovaCallable::NativeFunction(function) => {

                let mut source_index = argument_start;
                let source_end = argument_start + argument_number;

                let mut arguments = Vec::new();

                while source_index < source_end {
                    let object = self.package_register_into_nova_object(source_index);
                    arguments.push(object);
                    source_index += 1;
                }

                let result = (function.function)(arguments);

                if let Err(error) = result {
                    self.emit_error_with_message(&error);
                    return;
                }

                let result = result.unwrap();

                match result {
                    NovaObject::Float64(value) => {
                        let register = Register::new(RegisterValueKind::Float64, value.to_bits());
                        self.set_value_in_register(RegisterID::RRTN as Instruction, register);
                    }

                    NovaObject::Int64(value) => {
                        let register = Register::new(RegisterValueKind::Int64, value as u64);
                        self.set_value_in_register(RegisterID::RRTN as Instruction, register);
                    }

                    NovaObject::None => {
                        self.set_value_in_register(RegisterID::RRTN as Instruction, Register::empty());
                    }

                    _ => {
                        let memory_location = self.store_object_in_memory(result);
                        let register = Register::new(RegisterValueKind::MemAddress, memory_location as u64);
                        self.set_value_in_register(RegisterID::RRTN as Instruction, register);
                    }
                }
            }

            NovaCallable::None => {
                self.emit_error_with_message("Called a None Value");
            }
        }
    }

    #[inline(always)]
    fn return_none(&mut self) {
        self.set_value_in_register(RegisterID::RRTN as Instruction, Register::empty());
        self.drop_frame();
    }

    #[inline(always)]
    fn return_val(&mut self, instruction: Instruction) {
        let value_source = instruction_decoder::decode_source_register_1(instruction);
        let value_register = self.get_register(value_source);

        self.set_value_in_register(RegisterID::RRTN as Instruction, value_register);

        self.drop_frame();
    }

    #[inline(always)]
    fn load_return(&mut self, instruction: Instruction) {
        let destination = instruction_decoder::decode_destination_register(instruction);

        let return_register = self.registers[RegisterID::RRTN as usize];
        self.set_value_in_register(destination, return_register);
    }

    #[inline(always)]
    fn print(&self, instruction: Instruction) {
        let source = instruction_decoder::decode_source_register_1(instruction);
        let newline = instruction_decoder::decode_destination_register(instruction);

        let register = self.get_register(source);

        match register.kind {
            RegisterValueKind::Int64 => {
                print!("{}", register.value as i64)
            }
            RegisterValueKind::Float64 => {
                print!("{}", f64::from_bits(register.value))
            }
            RegisterValueKind::None => {
                print!("None")
            }
            RegisterValueKind::Bool => {
                print!("{}", register.value == 1)
            }
            RegisterValueKind::MemAddress => {
                let address = register.value;
                let object = self.load_object_from_memory(address);
                print!("{}", object);
            }

            RegisterValueKind::ImmAddress => {
                let immutable = &self.immutables[register.value as usize];
                print!("{}", immutable);
            }
        }
        if newline == 1 {
            println!()
        }
    }

    #[inline(always)]
    fn negate(&mut self, instruction: Instruction) {
        //let destination = InstructionDecoder::decode_destination_register(instruction);
        let source = instruction_decoder::decode_source_register_1(instruction);
        let destination = source; // negate value in place

        let register = self.get_register(source);
        if let RegisterValueKind::Float64 = register.kind {
            let value = f64::from_bits(register.value);
            let value = -value;
            let value = value.to_bits();

            let register = Register::new(RegisterValueKind::Float64, value);
            self.set_value_in_register(destination, register);
            return;
        }

        self.emit_error_with_message("Cannot negate non float32 value");
    }

    #[inline(always)]
    fn add(&mut self, instruction: Instruction) {
        let destination_register = instruction_decoder::decode_destination_register(instruction);
        let source_register_1 = instruction_decoder::decode_source_register_1(instruction);
        let source_register_2 = instruction_decoder::decode_source_register_2(instruction);

        let register_1 = self.get_register(source_register_1);
        let register_2 = self.get_register(source_register_2);

        if let (RegisterValueKind::Float64, RegisterValueKind::Float64) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = f64::from_bits(register_1.value);
            let value_2 = f64::from_bits(register_2.value);

            let sum = value_1 + value_2;
            let sum = sum.to_bits();

            let new_value = Register::new(register_1.kind, sum);
            self.set_value_in_register(destination_register, new_value);
            return;
        }

        if let (RegisterValueKind::Int64, RegisterValueKind::Int64) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = register_1.value as i64;
            let value_2 = register_2.value as i64;

            let sum = value_1 + value_2;
            let sum = sum as u64;

            let new_value = Register::new(register_1.kind, sum);
            self.set_value_in_register(destination_register, new_value);
            return;
        }

        if let (RegisterValueKind::Float64, RegisterValueKind::Int64) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = f64::from_bits(register_1.value);
            let value_2 = register_2.value as i64;

            let sum = value_1 + (value_2 as f64);
            let sum = sum.to_bits();

            let new_value = Register::new(register_1.kind, sum);
            self.set_value_in_register(destination_register, new_value);
            return;
        }

        if let (RegisterValueKind::Int64, RegisterValueKind::Float64) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = register_1.value as i64;
            let value_2 = f64::from_bits(register_2.value);

            let sum = value_1 as f64 + value_2;
            let sum = sum.to_bits();

            let new_value = Register::new(register_1.kind, sum);
            self.set_value_in_register(destination_register, new_value);
            return;
        }

        if let (RegisterValueKind::MemAddress, RegisterValueKind::Float64) =
            (register_1.kind, register_2.kind)
        {
            let object1 = self.load_object_from_memory(register_1.value);
            if let NovaObject::String(string) = object1 {
                let value2 = f64::from_bits(register_2.value);
                let value2 = value2.to_string();

                let mut new_value = *string.clone();
                new_value.push_str(&value2);

                let new_object = NovaObject::String(Box::new(new_value));
                let address = self.store_object_in_memory(new_object);
                self.load_memory_address_to_register(destination_register, address);
                return;
            }
            self.emit_error_with_message(&format!(
                "cannot add {:?} to {:?}",
                object1, register_2.kind
            ));
        }

        if let (RegisterValueKind::ImmAddress, RegisterValueKind::Float64) =
            (register_1.kind, register_2.kind)
        {
            let object1 = &self.immutables[register_1.value as usize];
            if let NovaObject::String(string) = object1 {
                let value2 = f64::from_bits(register_2.value);
                let value2 = value2.to_string();

                let mut new_value = *string.clone();
                new_value.push_str(&value2);

                let new_object = NovaObject::String(Box::new(new_value));
                let address = self.store_object_in_memory(new_object);
                self.load_memory_address_to_register(destination_register, address);
                return;
            }
            self.emit_error_with_message(&format!(
                "cannot add {:?} to {:?}",
                object1, register_2.kind
            ));
        }

        if let (RegisterValueKind::Float64, RegisterValueKind::MemAddress) =
            (register_1.kind, register_2.kind)
        {
            let object2 = self.load_object_from_memory(register_2.value);
            if let NovaObject::String(string) = object2 {
                let value1 = f64::from_bits(register_1.value);
                let value1 = value1.to_string();

                let mut new_value = value1;
                new_value.push_str(string);

                let new_object = NovaObject::String(Box::new(new_value));
                let address = self.store_object_in_memory(new_object);
                self.load_memory_address_to_register(destination_register, address);
                return;
            }
            self.emit_error_with_message(&format!(
                "cannot add {:?} to {:?}",
                register_1.kind, object2
            ));
        }

        if let (RegisterValueKind::Float64, RegisterValueKind::ImmAddress) =
            (register_1.kind, register_2.kind)
        {
            let object2 = &self.immutables[register_2.value as usize];
            if let NovaObject::String(string) = object2 {
                let value1 = f64::from_bits(register_1.value);
                let value1 = value1.to_string();

                let mut new_value = value1;
                new_value.push_str(string);

                let new_object = NovaObject::String(Box::new(new_value));
                let address = self.store_object_in_memory(new_object);
                self.load_memory_address_to_register(destination_register, address);
                return;
            }
            self.emit_error_with_message(&format!(
                "cannot add {:?} to {:?}",
                register_1.kind, object2
            ));
        }

        self.emit_error_with_message(&format!(
            "cannot add {:?} to {:?}",
            register_1.kind, register_2.kind
        ))
    }

    #[inline(always)]
    fn sub(&mut self, instruction: Instruction) {
        let destination_register = instruction_decoder::decode_destination_register(instruction);
        let source_register_1 = instruction_decoder::decode_source_register_1(instruction);
        let source_register_2 = instruction_decoder::decode_source_register_2(instruction);

        let register_1 = self.get_register(source_register_1);
        let register_2 = self.get_register(source_register_2);

        if let (RegisterValueKind::Float64, RegisterValueKind::Float64) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = f64::from_bits(register_1.value);
            let value_2 = f64::from_bits(register_2.value);

            let sub = value_1 - value_2;
            let sub = sub.to_bits();

            let new_value = Register::new(register_1.kind, sub);
            self.set_value_in_register(destination_register, new_value);
            return;
        }

        if let (RegisterValueKind::Int64, RegisterValueKind::Int64) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = register_1.value as i64;
            let value_2 = register_2.value as i64;

            let result = value_1 - value_2;
            let result = result as u64;

            let new_value = Register::new(register_1.kind, result);
            self.set_value_in_register(destination_register, new_value);
            return;
        }

        self.emit_error_with_message(&format!(
            "cannot subtract {:?} by {:?}",
            register_1.kind, register_2.kind
        ))
    }

    #[inline(always)]
    fn mul(&mut self, instruction: Instruction) {
        let destination_register = instruction_decoder::decode_destination_register(instruction);
        let source_register_1 = instruction_decoder::decode_source_register_1(instruction);
        let source_register_2 = instruction_decoder::decode_source_register_2(instruction);

        let register_1 = self.get_register(source_register_1);
        let register_2 = self.get_register(source_register_2);

        if let (RegisterValueKind::Float64, RegisterValueKind::Float64) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = f64::from_bits(register_1.value);
            let value_2 = f64::from_bits(register_2.value);

            let mul = value_1 * value_2;
            let mul = mul.to_bits();

            let new_value = Register::new(register_1.kind, mul);
            self.set_value_in_register(destination_register, new_value);
            return;
        }

        if let (RegisterValueKind::Int64, RegisterValueKind::Int64) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = register_1.value as i64;
            let value_2 = register_2.value as i64;

            let result = value_1 * value_2;
            let result = result as u64;

            let new_value = Register::new(register_1.kind, result);
            self.set_value_in_register(destination_register, new_value);
            return;
        }

        self.emit_error_with_message(&format!(
            "cannot multiply {:?} by {:?}",
            register_1.kind, register_2.kind
        ))
    }

    #[inline(always)]
    fn div(&mut self, instruction: Instruction) {
        let destination_register = instruction_decoder::decode_destination_register(instruction);
        let source_register_1 = instruction_decoder::decode_source_register_1(instruction);
        let source_register_2 = instruction_decoder::decode_source_register_2(instruction);

        let register_1 = self.get_register(source_register_1);
        let register_2 = self.get_register(source_register_2);

        if let (RegisterValueKind::Float64, RegisterValueKind::Float64) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = f64::from_bits(register_1.value);
            let value_2 = f64::from_bits(register_2.value);

            let div = value_1 / value_2;
            let div = div.to_bits();

            let new_value = Register::new(register_1.kind, div);
            self.set_value_in_register(destination_register, new_value);
            return;
        }

        if let (RegisterValueKind::Int64, RegisterValueKind::Int64) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = register_1.value as i64;
            let value_2 = register_2.value as i64;

            let result = value_1 as f64 / value_2 as f64;
            let result = result.to_bits();

            let new_value = Register::new(RegisterValueKind::Float64, result);
            self.set_value_in_register(destination_register, new_value);
            return;
        }

        self.emit_error_with_message(&format!(
            "cannot divide {:?} by {:?}",
            register_1.kind, register_2.kind
        ))
    }

    #[inline(always)]
    fn pow(&mut self, instruction: Instruction) {
        let destination_register = instruction_decoder::decode_destination_register(instruction);
        let source_register_1 = instruction_decoder::decode_source_register_1(instruction);
        let source_register_2 = instruction_decoder::decode_source_register_2(instruction);

        let register_1 = self.get_register(source_register_1);
        let register_2 = self.get_register(source_register_2);

        if let (RegisterValueKind::Float64, RegisterValueKind::Float64) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = f64::from_bits(register_1.value);
            let value_2 = f64::from_bits(register_2.value);

            let pow = value_1.powf(value_2);
            let pow = pow.to_bits();

            let new_value = Register::new(register_1.kind, pow);
            self.set_value_in_register(destination_register, new_value);
            return;
        }

        self.emit_error_with_message(&format!(
            "cannot calculate power of {:?} to {:?}",
            register_1.kind, register_2.kind
        ))
    }

    #[inline(always)]
    fn modulus(&mut self, instruction: Instruction) {
        let destination_register = instruction_decoder::decode_destination_register(instruction);
        let source_register_1 = instruction_decoder::decode_source_register_1(instruction);
        let source_register_2 = instruction_decoder::decode_source_register_2(instruction);

        let register_1 = self.get_register(source_register_1);
        let register_2 = self.get_register(source_register_2);

        if let (RegisterValueKind::Float64, RegisterValueKind::Float64) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = f64::from_bits(register_1.value) as i32;
            let value_2 = f64::from_bits(register_2.value) as i32;

            let modulus = (value_1 % value_2) as f64;
            let modulus = modulus.to_bits();

            let new_value = Register::new(register_1.kind, modulus);
            self.set_value_in_register(destination_register, new_value);
            return;
        }

        self.emit_error_with_message(&format!(
            "cannot find modulus {:?} by {:?}",
            register_1.kind, register_2.kind
        ))
    }

    #[inline(always)]
    /// compares if first register is less than second register
    fn less(&mut self, instruction: Instruction) {
        let source1 = instruction_decoder::decode_source_register_1(instruction);
        let source2 = instruction_decoder::decode_source_register_2(instruction);

        let destination = instruction_decoder::decode_destination_register(instruction);

        let register1 = self.get_register(source1);
        let register2 = self.get_register(source2);

        let less = self.compare_registers(OpCode::Less, register1, register2);
        if self.check_error() {
            return;
        }

        let register = Register {
            value: if less { 1 } else { 0 },
            kind: RegisterValueKind::Bool,
        };

        self.set_value_in_register(destination, register);
    }

    #[inline(always)]
    /// compares if first register is less than or equal to second register
    fn less_or_equal(&mut self, instruction: Instruction) {
        let source1 = instruction_decoder::decode_source_register_1(instruction);
        let source2 = instruction_decoder::decode_source_register_2(instruction);

        let destination = instruction_decoder::decode_destination_register(instruction);

        let register1 = self.get_register(source1);
        let register2 = self.get_register(source2);

        let less = self.compare_registers(OpCode::LessEqual, register1, register2);
        if self.check_error() {
            return;
        }

        let register = Register {
            value: if less { 1 } else { 0 },
            kind: RegisterValueKind::Bool,
        };

        self.set_value_in_register(destination, register);
    }

    #[inline(always)]
    /// compares if first register is less than or equal to second register
    fn equal(&mut self, instruction: Instruction) {
        let source1 = instruction_decoder::decode_source_register_1(instruction);
        let source2 = instruction_decoder::decode_source_register_2(instruction);

        let destination = instruction_decoder::decode_destination_register(instruction);

        let register1 = self.get_register(source1);
        let register2 = self.get_register(source2);

        let equal = self.compare_registers(OpCode::Equal, register1, register2);
        if self.check_error() {
            return;
        }

        let register = Register {
            value: if equal { 1 } else { 0 },
            kind: RegisterValueKind::Bool,
        };

        self.set_value_in_register(destination, register);
    }

    #[inline(always)]
    fn not(&mut self, instruction: Instruction) {
        let source = instruction_decoder::decode_source_register_1(instruction);
        let mut register = self.get_register(source);

        let is_true = self.is_truthy(register);
        register.value = if is_true { 0 } else { 1 };
        self.set_value_in_register(source, register);
    }

    #[inline(always)]
    fn is_truthy(&self, register: Register) -> bool {
        match register.kind {
            RegisterValueKind::None => false,
            RegisterValueKind::Int64 => true,
            RegisterValueKind::Float64 => true,
            RegisterValueKind::Bool => register.value == 1,
            RegisterValueKind::MemAddress => true,
            RegisterValueKind::ImmAddress => true,
        }
    }

    #[inline(always)]
    fn jump_if_false(&mut self, instruction: Instruction) {
        let source = instruction_decoder::decode_source_register_1(instruction);

        let register = self.get_register(source);
        let truthy = self.is_truthy(register);

        let jump_instruction = self.get_next_instruction();

        if !truthy {
            self.jump(jump_instruction);
        }
    }

    #[inline(always)]
    fn jump(&mut self, instruction: Instruction) {
        let offset = instruction_decoder::decode_immutable_address_small(instruction);
        let direction = instruction_decoder::decode_destination_register(instruction);
        if direction == 0 {
            self.registers[RegisterID::RPC as usize].value -= offset as u64 + 1; // backward jump, add one since the intepreter will automatically add 1 after instruction
        } else {
            self.registers[RegisterID::RPC as usize].value += offset as u64 - 1; // forward jump, minus one since the intepreter will automatically add 1 after instruction
        }
    }

    #[inline(always)]
    fn package_register_into_nova_object(&self, register_address: Instruction) -> NovaObject {
        let register = self.get_register(register_address);

        let value = match register.kind {
            RegisterValueKind::Int64 => NovaObject::Int64(register.value as i64),
            RegisterValueKind::Float64 => NovaObject::Float64(f64::from_bits(register.value)),
            RegisterValueKind::None => NovaObject::None,
            RegisterValueKind::MemAddress => self.load_object_from_memory(register.value).clone(),
            RegisterValueKind::ImmAddress => self.immutables[register.value as usize].clone(),
            RegisterValueKind::Bool => todo!()
        };

        value
    }

    #[inline(always)]
    fn load_constant_to_register(&mut self, instruction: Instruction) {
        let destination_register = instruction_decoder::decode_destination_register(instruction);
        let immutable_address = instruction_decoder::decode_immutable_address_small(instruction);

        let register = Register {
            kind: RegisterValueKind::ImmAddress,
            value: immutable_address as u64,
        };
        self.set_value_in_register(destination_register, register);
    }

    #[inline(always)]
    fn load_float32_to_register(&mut self, instruction: Instruction) {
        let destination_register = instruction_decoder::decode_destination_register(instruction);

        let number = self.get_next_instruction();
        let number = f32::from_bits(number);
        self.load_f64_to_register(destination_register, number as f64);
    }

    #[inline(always)]
    fn load_float64_to_register(&mut self, instruction: Instruction) {
        let destination_register = instruction_decoder::decode_destination_register(instruction);

        let first_half = self.get_next_instruction();
        let second_half = self.get_next_instruction();
        let number = instruction_decoder::merge_u32s(first_half, second_half);
        let number = f64::from_bits(number);
        self.load_f64_to_register(destination_register, number);
    }

    #[inline(always)]
    fn load_int32_to_register(&mut self, instruction: Instruction) {
        let destination_register = instruction_decoder::decode_destination_register(instruction);

        let number = self.get_next_instruction();
        let number = number as i32;
        self.load_i64_to_register(destination_register, number as i64);
    }

    #[inline(always)]
    fn load_int64_to_register(&mut self, instruction: Instruction) {
        let destination_register = instruction_decoder::decode_destination_register(instruction);

        let first_half = self.get_next_instruction();
        let second_half = self.get_next_instruction();
        let number = instruction_decoder::merge_u32s(first_half, second_half);
        let number = number as i64;
        self.load_i64_to_register(destination_register, number);
    }

    #[inline(always)]
    fn load_nil_to_register(&mut self, instruction: Instruction) {
        let destination = instruction_decoder::decode_destination_register(instruction);
        let register = Register::new(RegisterValueKind::None, 0);
        self.set_value_in_register(destination, register);
    }

    #[inline(always)]
    fn load_memory_address_to_register(&mut self, destination: Instruction, address: Instruction) {
        let value = Register::new(RegisterValueKind::MemAddress, address as u64);
        self.set_value_in_register(destination, value);
    }

    #[inline(always)]
    fn load_f64_to_register(&mut self, destination: Instruction, number: f64) {
        let number = number.to_bits();
        let register = Register::new(RegisterValueKind::Float64, number);
        self.set_value_in_register(destination, register);
    }

    #[inline(always)]
    fn load_i64_to_register(&mut self, destination: Instruction, number: i64) {
        let number = number as u64;
        let register = Register::new(RegisterValueKind::Int64, number);
        self.set_value_in_register(destination, register);
    }

    #[inline(always)]
    fn load_bool_to_register(&mut self, instruction: Instruction) {
        let destination = instruction_decoder::decode_destination_register(instruction);
        let boolean = instruction_decoder::decode_immutable_address_small(instruction);
        let register = Register::new(RegisterValueKind::Float64, boolean as u64);
        self.set_value_in_register(destination, register);
    }

    #[inline(always)]
    fn emit_error_with_message(&mut self, message: &str) {
        let address = self.store_object_in_memory(NovaObject::String(Box::new(message.to_string())));
        self.load_memory_address_to_register(RegisterID::RERR as Instruction, address);
    }

    #[inline(always)]
    fn check_error(&self) -> bool {
        let register = self.get_register(RegisterID::RERR as Instruction);

        if let RegisterValueKind::MemAddress = register.kind {
            return true;
        }

        false
    }

    #[inline(always)]
    fn print_error(&self) {
        let register = self.get_register(RegisterID::RERR as Instruction);

        if let RegisterValueKind::MemAddress = register.kind {
            let address = register.value;
            let object = &self.memory[address as usize];
            eprint!("Error: ");

            if let NovaObject::String(string) = object {
                eprint!("{}", string)
            }
            eprintln!();
        }
    }

    /// store a NovaObject in the memory and return its allocated address
    #[inline(always)]
    fn store_object_in_memory(&mut self, object: NovaObject) -> Instruction {
        self.memory.push(object);
        let address = self.memory.len() - 1;
        address as Instruction
    }

    #[inline(always)]
    fn get_next_instruction(&mut self) -> Instruction {
        let instruction = self.peek_next_instruction();
        unsafe {
            self.registers.get_unchecked_mut(RegisterID::RPC as usize).value += 1;
        }
        
        instruction
    }

    #[inline(always)]
    fn get_register(&self, register_id: Instruction) -> Register {
        unsafe {
            return *self.registers.get_unchecked(register_id as usize);
        }
    }

    #[inline(always)]
    fn set_value_in_register(&mut self, register_id: Instruction, value: Register) {
        unsafe {
            let register = self.registers.get_unchecked_mut(register_id as usize);
            register.kind = value.kind;
            register.value = value.value;
        }
    }

    /// load an object from memory given the memory location
    #[inline(always)]
    fn load_object_from_memory(&self, address: u64) -> &NovaObject {
        unsafe {
            let object = self.memory.get_unchecked(address as usize);
            return object;
        }
        // &self.memory[address as usize]
    }


    #[inline(always)]
    fn free_memory_location(&mut self, _address: Instruction) {
        todo!()
    }

    /// defines an empty global variable in the virtual machine
    #[inline(always)]
    fn define_global_indirect(&mut self, instruction: Instruction) {
        let index = instruction_decoder::decode_immutable_address_small(instruction);
        let immutable = self.immutables[index as usize].clone();

        if let NovaObject::String(name) = immutable {
            let global_location = self.allocate_global();
            self.create_global(name.to_string(), global_location);
        }
    }

    /// bind a global location to an identifier
    #[inline(always)]
    fn create_global(&mut self, name: String, global_location: Instruction) {
        self.identifiers.insert(name, global_location);
    }

    /// allocate memory on the globals vector
    #[inline(always)]
    fn allocate_global(&mut self) -> Instruction {
        self.globals.push(Register::default());
        (self.globals.len() - 1) as Instruction
    }

    /// set value of a specified global location
    #[inline(always)]
    fn set_global_value(&mut self, address: Instruction, new_value: Register) {
        let current_value = self.globals[address as usize];

        if current_value.kind.is_mem_address() {
            self.free_memory_location(current_value.value as u32);
        }

        self.globals[address as usize] = new_value;
    }

    /// load value from a specified global address
    #[inline(always)]
    fn load_global_value(&mut self, destination: Instruction, global_address: Instruction) {
        let value = self.globals[global_address as usize];
        self.registers[destination as usize] = value;
    }

    /// store a value in a global location by first looking up name in the immutables array
    #[inline(always)]
    fn store_global_indirect(&mut self, instruction: Instruction) {
        let source = instruction_decoder::decode_source_register_1(instruction);
        let index = instruction_decoder::decode_immutable_address_small(instruction);

        let immutable = self.immutables[index as usize].clone();

        if let NovaObject::String(name) = immutable {
            let global_address = self.identifiers.get(name.as_str());

            if let Some(&address) = global_address {
                let register = self.get_register(source);
                self.set_global_value(address, register);
                self.clear_register(source);

                return;
            }

            self.emit_error_with_message(&format!("Cannot find global named: {}", name));
            self.clear_register(source);
            return;
        }

        self.emit_error_with_message(&format!("Invalid global identifier: {:?}", immutable));
        self.clear_register(source)
    }

    /// load a value from a global location into a register by first looking up its name in the immutables array
    #[inline(always)]
    fn load_global_indirect(&mut self, instruction: Instruction) {
        let destination = instruction_decoder::decode_destination_register(instruction);
        let index = instruction_decoder::decode_immutable_address_small(instruction);

        let immutable = self.immutables[index as usize].clone();

        if let NovaObject::String(name) = immutable {
            let global_address = self.identifiers.get(name.as_str());

            if let Some(&address) = global_address {
                self.load_global_value(destination, address);

                return;
            }

            self.emit_error_with_message(&format!("Cannot find global named: {}", name));
            return;
        }

        self.emit_error_with_message(&format!("Invalid global identifier: {:?}", immutable));
    }

    #[inline(always)]
    fn allocate_locals(&mut self, instruction: Instruction) {
        // number of variables
        let number = instruction_decoder::decode_immutable_address_small(instruction);
        let mut local_space = vec![Register::default(); number as usize];

        self.locals.append(&mut local_space)
    }

    #[inline(always)]
    fn allocate_local_variables(&mut self, number_of_locals: Instruction) {
        let mut local_space = vec![Register::default(); number_of_locals as usize];
        self.locals.append(&mut local_space)
    }

    #[inline(always)]
    fn deallocate_locals(&mut self, instruction: Instruction) {
        // number of variables
        let mut number = instruction_decoder::decode_immutable_address_small(instruction);

        while number > 0 {
            number -= 1;
            self.locals.pop();
        }
    }

    #[inline(always)]
    fn deallocate_local_variables(&mut self, number_of_locals: Instruction) {
        let mut number = number_of_locals;

        while number > 0 {
            number -= 1;
            self.locals.pop();
        }
    }

    #[inline(always)]
    fn store_local(&mut self, instruction: Instruction) {
        let source = instruction_decoder::decode_source_register_1(instruction);
        let address = instruction_decoder::decode_immutable_address_small(instruction);

        let register = self.get_register(source);
        let local_offset = self.registers[RegisterID::RLO as usize].value;
        self.locals[(address + local_offset as u32) as usize] = register;
        self.clear_register(source);
    }

    #[inline(always)]
    fn load_local(&mut self, instruction: Instruction) {
        let destination = instruction_decoder::decode_destination_register(instruction);
        let address = instruction_decoder::decode_immutable_address_small(instruction);

        let register = unsafe {
            let local_offset = self.get_register(RegisterID::RLO as u32).value;
            *self.locals.get_unchecked((address as u64 + local_offset) as usize)
        };
        
        self.set_value_in_register(destination, register);
    }

    #[inline(always)]
    fn clear_register(&mut self, register_id: Instruction) {
        let register = unsafe {
            self.registers.get_unchecked_mut(register_id as usize)
        };

        register.kind = RegisterValueKind::None;
        register.value = 0;
    }

    #[inline(always)]
    fn compare_registers(&mut self, op: OpCode, first: Register, second: Register) -> bool {
        match op {
            OpCode::Less => {
                if first.kind.is_float64() && second.kind.is_float64() {
                    let first = f64::from_bits(first.value);
                    let second = f64::from_bits(second.value);
                    return first < second;
                }

                if first.kind.is_int64() && second.kind.is_int64() {
                    let first = first.value as i64;
                    let second = second.value as i64;
                    return first < second;
                }

                if first.kind.is_int64() && second.kind.is_float64() {
                    let first = first.value as i64;
                    let second = f64::from_bits(second.value);
                    return (first as f64) < second;
                }

                if first.kind.is_float64() && second.kind.is_int64() {
                    let first = f64::from_bits(first.value);
                    let second = second.value as i64;
                    return first < (second as f64);
                }

                if first.kind.is_mem_address() && second.kind.is_mem_address() {
                    let first = self.load_object_from_memory(first.value);
                    let second = self.load_object_from_memory(second.value);

                    return first < second;
                }

                if first.kind.is_imm_address() && second.kind.is_imm_address() {
                    let first = &self.immutables[first.value as usize];
                    let second = &self.immutables[second.value as usize];

                    return first < second;
                }

                if first.kind.is_imm_address() && second.kind.is_mem_address() {
                    let first = &self.immutables[first.value as usize];
                    let second = self.load_object_from_memory(second.value);

                    return first < second;
                }

                if first.kind.is_mem_address() && second.kind.is_imm_address() {
                    let first = self.load_object_from_memory(first.value);
                    let second = &self.immutables[second.value as usize];

                    return first < second;
                }

                self.emit_error_with_message(&format!(
                    "cannot compare {:?} to {:?}",
                    first.kind, second.kind
                ));
            }

            OpCode::LessEqual => {
                if first.kind.is_float64() && second.kind.is_float64() {
                    let first = f64::from_bits(first.value);
                    let second = f64::from_bits(second.value);
                    return first <= second;
                }

                if first.kind.is_int64() && second.kind.is_int64() {
                    let first = first.value as i64;
                    let second = second.value as i64;
                    return first <= second;
                }

                if first.kind.is_int64() && second.kind.is_float64() {
                    let first = first.value as i64;
                    let second = f64::from_bits(second.value);
                    return (first as f64) <= second;
                }

                if first.kind.is_float64() && second.kind.is_int64() {
                    let first = f64::from_bits(first.value);
                    let second = second.value as i64;
                    return first <= (second as f64);
                }

                if first.kind.is_mem_address() && second.kind.is_mem_address() {
                    let first = self.load_object_from_memory(first.value);
                    let second = self.load_object_from_memory(second.value);

                    return first <= second;
                }

                if first.kind.is_imm_address() && second.kind.is_mem_address() {
                    let first = &self.immutables[first.value as usize];
                    let second = self.load_object_from_memory(second.value);

                    return first <= second;
                }

                if first.kind.is_mem_address() && second.kind.is_imm_address() {
                    let first = self.load_object_from_memory(first.value);
                    let second = &self.immutables[second.value as usize];

                    return first <= second;
                }

                self.emit_error_with_message(&format!(
                    "cannot compare {:?} to {:?}",
                    first.kind, second.kind
                ));
            }

            OpCode::Equal => {

                if first.kind.is_int64() && second.kind.is_float64() {
                    let first = first.value as i64;
                    let second = f64::from_bits(second.value);
                    return (first as f64) == second;
                }

                if first.kind.is_float64() && second.kind.is_int64() {
                    let first = f64::from_bits(first.value);
                    let second = second.value as i64;
                    return first == (second as f64);
                }
                
                if first.kind != second.kind {
                    return false;
                }

                if first.kind.is_none() && second.kind.is_none() {
                    return true;
                }

                if first.kind.is_float64() && second.kind.is_float64() {
                    return first.value == second.value;
                }

                if first.kind.is_int64() && second.kind.is_int64() {
                    return first.value == second.value;
                }

                if first.kind.is_mem_address() && second.kind.is_mem_address() {
                    let first = self.load_object_from_memory(first.value);
                    let second = self.load_object_from_memory(second.value);

                    return first == second;
                }

                if first.kind.is_imm_address() && second.kind.is_mem_address() {
                    let first = &self.immutables[first.value as usize];
                    let second = self.load_object_from_memory(second.value);

                    return first == second;
                }

                if first.kind.is_mem_address() && second.kind.is_imm_address() {
                    let first = self.load_object_from_memory(first.value);
                    let second = &self.immutables[second.value as usize];

                    return first == second;
                }

                self.emit_error_with_message(&format!(
                    "cannot compare {:?} to {:?}",
                    first.kind, second.kind
                ));
            }

            _ => {
                self.emit_error_with_message(&format!(
                    "Undefined comparison operator {:#x}",
                    op as Instruction
                ));
            }
        }

        false
    }

    #[inline(always)]
    fn peek_next_instruction(&self) -> Instruction {
        unsafe {
            let instruction_pointer = self.registers.get_unchecked(RegisterID::RPC as usize).value as usize;
            return *self.instructions.get_unchecked(instruction_pointer);
        }
    }

    #[cfg(feature = "debug")]
    fn debug(&self) {
        #[cfg(feature = "dbg_code")]
        {
            let current = self.registers[RegisterID::RPC as usize].value;
            println!(
                "[{}]: {}",
                current,
                debug_instruction(
                    &self.instructions,
                    self.registers[RegisterID::RPC as usize].value,
                )
            );
        }
        #[cfg(feature = "verbose")]
        self.print_register_values();
        #[cfg(feature = "dbg_global")]
        self.print_globals();
        #[cfg(feature = "dbg_local")]
        self.print_locals();
        #[cfg(feature = "dbg_global")]
        self.print_identifiers();
        #[cfg(feature = "dbg_memory")]
        self.print_memory();
        #[cfg(feature = "verbose")]
        Self::wait_for_input();
    }

    #[cfg(feature = "debug")]
    fn wait_for_input() {
        use std::io;

        let mut buffer = String::new();
        let _ = io::stdin().read_line(&mut buffer);
    }

    #[cfg(feature = "verbose")]
    fn print_register_values(&self) {
        println!("{:=^30}", "Registers");
        for register_index in 0..RegisterID::RMax as usize + 1 {
            let register = self.get_register(register_index as u32);
            println!("==> R{:<2}: {}", register_index, register);
        }
        println!("{:=^30}", "");
    }

    #[cfg(feature = "dbg_memory")]
    fn print_memory(&self) {
        println!("{:=^30}", "Heap");
        println!("==> {:?}", &self.memory);
        println!("{:=^30}", "");
    }

    #[cfg(feature = "dbg_global")]
    fn print_globals(&self) {
        println!("{:=^30}", "Globals");
        print_vec_of_registers(&self.globals);
        println!("{:=^30}", "");
    }

    #[cfg(feature = "dbg_local")]
    fn print_locals(&self) {
        println!("{:=^30}", "Locals");
        print_vec_of_registers(&self.locals);
        println!("{:=^30}", "");
    }

    #[cfg(feature = "dbg_global")]
    fn print_identifiers(&self) {
        println!("{:=^30}", "Identifiers");
        println!("==> {:?}", &self.identifiers);
        println!("{:=^30}", "");
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
