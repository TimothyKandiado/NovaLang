use crate::{
    bytecode::OpCode,
    frame::Frame,
    instruction::{Instruction, InstructionDecoder},
    object::{MappedMemory, NovaObject, RegisterValueKind},
    program::Program,
    register::{Register, RegisterID},
};

#[cfg(feature = "debug")]
use crate::debug::debug_instruction;

const PC_START: Instruction = 0x0;

pub struct VirtualMachine {
    instructions: Vec<Instruction>,
    immutables: Vec<NovaObject>,
    registers: [Register; RegisterID::RMax as usize + 1],
    running: bool,
    instruction_count: usize,
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
            instruction_count: 0,
            memory: Vec::new(),
            frames: vec![Frame::main()],
            locals: Vec::new(),
            globals: Vec::new(),
            identifiers: MappedMemory::new(),
        }
    }

    pub fn load_program(&mut self, program: Program) {
        for instruction in &program.instructions {
            self.instruction_count += 1;
            self.instructions.push(*instruction);
        }

        for immutable in &program.immutables {
            self.immutables.push(immutable.clone());
        }
    }

    pub fn start_vm(&mut self, offset: Instruction) {
        self.running = true;
        self.registers[RegisterID::RPC as usize].value = offset + PC_START;

        while self.running {
            #[cfg(feature = "debug")]
            self.debug();

            let instruction = self.get_next_instruction();

            self.execute_instruction(instruction);

            if self.check_error() {
                self.print_error();
                break;
            }
        }
    }

    fn execute_instruction(&mut self, instruction: Instruction) {
        let opcode = InstructionDecoder::decode_opcode(instruction);

        match opcode {
            // System Interrupt
            x if x == OpCode::Halt as u32 => {
                self.running = false;
            }

            // Unary operations
            x if x == OpCode::Neg as u32 => {
                self.negate(instruction);
            }

            // Binary Operations
            x if x == OpCode::Add as u32 => {
                self.add(instruction);
            }
            x if x == OpCode::Sub as u32 => {
                self.sub(instruction);
            }
            x if x == OpCode::Mul as u32 => {
                self.mul(instruction);
            }
            x if x == OpCode::Div as u32 => {
                self.div(instruction);
            }
            x if x == OpCode::Pow as u32 => {
                self.pow(instruction);
            }
            x if x == OpCode::Mod as u32 => {
                self.modulus(instruction);
            }

            // Register Manipulation
            x if x == OpCode::LoadK as u32 => {
                self.load_constant_to_register(instruction);
            }

            x if x == OpCode::LoadNil as u32 => {
                self.load_nil_to_register(instruction);
            }

            x if x == OpCode::LoadBool as u32 => {
                self.load_bool_to_register(instruction);
            }

            x if x == OpCode::LoadFloat as u32 => {
                self.load_float32_to_register(instruction);
            }

            x if x == OpCode::Move as u32 => {
                self.move_register(instruction);
            }

            // Variable Manipulation
            x if x == OpCode::DefineGlobalIndirect as u32 => {
                self.define_global_indirect(instruction);
            }

            x if x == OpCode::StoreGlobalIndirect as u32 => {
                self.store_global_indirect(instruction);
            }

            x if x == OpCode::LoadGlobalIndirect as u32 => {
                self.load_global_indirect(instruction);
            }
            

            x if x == OpCode::LoadGlobal as u32 => {
                let destination = InstructionDecoder::decode_destination_register(instruction);
                let address = InstructionDecoder::decode_immutable_address_small(instruction);

                self.load_global_value(destination, address);
            }

            x if x == OpCode::AllocateLocal as u32 => {
                self.allocate_locals(instruction);
            }

            x if x == OpCode::DeallocateLocal as u32 => {
                self.deallocate_locals(instruction);
            }

            x if x == OpCode::StoreLocal as u32 => {
                self.store_local(instruction);
            }

            x if x == OpCode::LoadLocal as u32 => {
                self.load_local(instruction);
            }

            // Control flow
            x if x == OpCode::LessJump as u32 => {
                self.less_or_jump(instruction);
            }

            x if x == OpCode::Jump as u32 => {
                self.jump(instruction);
            }

            x if x == OpCode::NewFrame as u32 => self.new_frame(),

            x if x == OpCode::Return as u32 => self.drop_frame(),

            // IO
            x if x == OpCode::Print as u32 => {
                self.print(instruction);
            }

            _ => self.emit_error_with_message(&format!(
                "Unsupported opcode instruction ({:#x})",
                opcode
            )),
        }
    }
    #[inline(always)]
    fn move_register(&mut self, instruction: Instruction) {
        let destination = InstructionDecoder::decode_destination_register(instruction);
        let source = InstructionDecoder::decode_source_register_1(instruction);

        let value = self.get_register(source);
        self.set_value_in_register(destination, value);
    }

    #[inline(always)]
    fn new_frame(&mut self) {
        let return_address = self.registers[RegisterID::RPC as usize].value;
        let local_offset = self.registers[RegisterID::RLO as usize].value;
        let frame = Frame::new(return_address, local_offset, false);

        self.frames.push(frame);
        self.increase_local_offset();
    }

    #[inline(always)]
    fn increase_local_offset(&mut self) {
        self.registers[RegisterID::RLO as usize].value += (self.locals.len()) as Instruction;
    }

    #[inline(always)]
    fn drop_frame(&mut self) {
        let frame = self.frames.pop();

        if let Some(frame) = frame {
            if frame.is_main {
                self.running = false;
                return;
            }
            self.registers[RegisterID::RPC as usize].value = frame.return_address + 1;
            self.registers[RegisterID::RLO as usize].value = frame.local_offset;
        } else {
            self.running = false;
        }
    }

    #[inline(always)]
    fn print(&self, instruction: Instruction) {
        let source = InstructionDecoder::decode_source_register_1(instruction);
        let newline = InstructionDecoder::decode_destination_register(instruction);

        let register = self.get_register(source);

        match register.kind {
            RegisterValueKind::Float32 => {
                print!("{}", f32::from_bits(register.value))
            }
            RegisterValueKind::None => {
                print!("None")
            }
            RegisterValueKind::Bool => {
                print!("{}", register.value == 1)
            }
            RegisterValueKind::MemAddress => {
                let address = register.value;
                let object = self.get_object_from_memory(address);
                print!("{:?}", object);
            }

            RegisterValueKind::ImmAddress => {
                let immutable = &self.immutables[register.value as usize];
                print!("{:?}", immutable);
            }
        }
        if newline == 1 {
            println!()
        }
    }

    #[inline(always)]
    fn negate(&mut self, instruction: Instruction) {
        //let destination = InstructionDecoder::decode_destination_register(instruction);
        let source = InstructionDecoder::decode_source_register_1(instruction);
        let destination = source; // negate value in place

        let register = self.get_register(source);
        if let RegisterValueKind::Float32 = register.kind {
            let value = f32::from_bits(register.value);
            let value = -value;
            let value = value.to_bits();

            let register = Register::new(RegisterValueKind::Float32, value);
            self.set_value_in_register(destination, register);
            return;
        }

        self.emit_error_with_message("Cannot negate non float32 value");
    }

    #[inline(always)]
    fn add(&mut self, instruction: Instruction) {
        let destination_register = InstructionDecoder::decode_destination_register(instruction);
        let source_register_1 = InstructionDecoder::decode_source_register_1(instruction);
        let source_register_2 = InstructionDecoder::decode_source_register_2(instruction);

        let register_1 = self.get_register(source_register_1);
        let register_2 = self.get_register(source_register_2);

        if let (RegisterValueKind::Float32, RegisterValueKind::Float32) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = f32::from_bits(register_1.value);
            let value_2 = f32::from_bits(register_2.value);

            let sum = value_1 + value_2;
            let sum = sum.to_bits();

            let new_value = Register::new(register_1.kind, sum);
            self.set_value_in_register(destination_register, new_value);
            return;
        }

        if let (RegisterValueKind::MemAddress, RegisterValueKind::Float32) =
            (register_1.kind, register_2.kind)
        {
            let object1 = self.get_object_from_memory(register_1.value);
            if let NovaObject::String(string) = object1 {
                let value2 = f32::from_bits(register_2.value);
                let value2 = value2.to_string();

                let mut new_value = *string.clone();
                new_value.push_str(&value2);

                let new_object = NovaObject::String(Box::new(new_value));
                let address = self.load_object_to_memory(new_object);
                self.load_memory_address_to_register(destination_register, address);
                return;
            }
            self.emit_error_with_message(&format!(
                "cannot add {:?} to {:?}",
                object1, register_2.kind
            ));
        }

        if let (RegisterValueKind::ImmAddress, RegisterValueKind::Float32) =
            (register_1.kind, register_2.kind)
        {
            let object1 = &self.immutables[register_1.value as usize];
            if let NovaObject::String(string) = object1 {
                let value2 = f32::from_bits(register_2.value);
                let value2 = value2.to_string();

                let mut new_value = *string.clone();
                new_value.push_str(&value2);

                let new_object = NovaObject::String(Box::new(new_value));
                let address = self.load_object_to_memory(new_object);
                self.load_memory_address_to_register(destination_register, address);
                return;
            }
            self.emit_error_with_message(&format!(
                "cannot add {:?} to {:?}",
                object1, register_2.kind
            ));
        }

        if let (RegisterValueKind::Float32, RegisterValueKind::MemAddress) =
            (register_1.kind, register_2.kind)
        {
            let object2 = self.get_object_from_memory(register_2.value);
            if let NovaObject::String(string) = object2 {
                let value1 = f32::from_bits(register_1.value);
                let value1 = value1.to_string();

                let mut new_value = value1;
                new_value.push_str(string);

                let new_object = NovaObject::String(Box::new(new_value));
                let address = self.load_object_to_memory(new_object);
                self.load_memory_address_to_register(destination_register, address);
                return;
            }
            self.emit_error_with_message(&format!(
                "cannot add {:?} to {:?}",
                register_1.kind, object2
            ));
        }

        if let (RegisterValueKind::Float32, RegisterValueKind::ImmAddress) =
            (register_1.kind, register_2.kind)
        {
            let object2 = &self.immutables[register_2.value as usize];
            if let NovaObject::String(string) = object2 {
                let value1 = f32::from_bits(register_1.value);
                let value1 = value1.to_string();

                let mut new_value = value1;
                new_value.push_str(&string);

                let new_object = NovaObject::String(Box::new(new_value));
                let address = self.load_object_to_memory(new_object);
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
        let destination_register = InstructionDecoder::decode_destination_register(instruction);
        let source_register_1 = InstructionDecoder::decode_source_register_1(instruction);
        let source_register_2 = InstructionDecoder::decode_source_register_2(instruction);

        let register_1 = self.get_register(source_register_1);
        let register_2 = self.get_register(source_register_2);

        if let (RegisterValueKind::Float32, RegisterValueKind::Float32) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = f32::from_bits(register_1.value);
            let value_2 = f32::from_bits(register_2.value);

            let sub = value_1 - value_2;
            let sub = sub.to_bits();

            let new_value = Register::new(register_1.kind, sub);
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
        let destination_register = InstructionDecoder::decode_destination_register(instruction);
        let source_register_1 = InstructionDecoder::decode_source_register_1(instruction);
        let source_register_2 = InstructionDecoder::decode_source_register_2(instruction);

        let register_1 = self.get_register(source_register_1);
        let register_2 = self.get_register(source_register_2);

        if let (RegisterValueKind::Float32, RegisterValueKind::Float32) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = f32::from_bits(register_1.value);
            let value_2 = f32::from_bits(register_2.value);

            let mul = value_1 * value_2;
            let mul = mul.to_bits();

            let new_value = Register::new(register_1.kind, mul);
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
        let destination_register = InstructionDecoder::decode_destination_register(instruction);
        let source_register_1 = InstructionDecoder::decode_source_register_1(instruction);
        let source_register_2 = InstructionDecoder::decode_source_register_2(instruction);

        let register_1 = self.get_register(source_register_1);
        let register_2 = self.get_register(source_register_2);

        if let (RegisterValueKind::Float32, RegisterValueKind::Float32) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = f32::from_bits(register_1.value);
            let value_2 = f32::from_bits(register_2.value);

            let div = value_1 / value_2;
            let div = div.to_bits();

            let new_value = Register::new(register_1.kind, div);
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
        let destination_register = InstructionDecoder::decode_destination_register(instruction);
        let source_register_1 = InstructionDecoder::decode_source_register_1(instruction);
        let source_register_2 = InstructionDecoder::decode_source_register_2(instruction);

        let register_1 = self.get_register(source_register_1);
        let register_2 = self.get_register(source_register_2);

        if let (RegisterValueKind::Float32, RegisterValueKind::Float32) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = f32::from_bits(register_1.value);
            let value_2 = f32::from_bits(register_2.value);

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
        let destination_register = InstructionDecoder::decode_destination_register(instruction);
        let source_register_1 = InstructionDecoder::decode_source_register_1(instruction);
        let source_register_2 = InstructionDecoder::decode_source_register_2(instruction);

        let register_1 = self.get_register(source_register_1);
        let register_2 = self.get_register(source_register_2);

        if let (RegisterValueKind::Float32, RegisterValueKind::Float32) =
            (register_1.kind, register_2.kind)
        {
            let value_1 = f32::from_bits(register_1.value) as i32;
            let value_2 = f32::from_bits(register_2.value) as i32;

            let modulus = (value_1 % value_2) as f32;
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
    /// compares if first register is less than second register, then jumps if comparison false
    fn less_or_jump(&mut self, instruction: Instruction) {
        let source1 = InstructionDecoder::decode_source_register_1(instruction);
        let source2 = InstructionDecoder::decode_source_register_2(instruction);

        let register1 = self.get_register(source1);
        let register2 = self.get_register(source2);

        let less = self.compare_registers(OpCode::LessJump, register1, register2);
        if self.check_error() {
            return;
        }

        let next = self.get_next_instruction();

        if !less {
            self.jump(next)
        }
    }

    #[inline(always)]
    fn jump(&mut self, instruction: Instruction) {
        let offset = InstructionDecoder::decode_immutable_address_small(instruction);
        let direction = InstructionDecoder::decode_destination_register(instruction);
        if direction == 0 {
            self.registers[RegisterID::RPC as usize].value -= offset + 1; // backward jump, add one since the intepreter will automatically add 1 after instruction
        } else {
            self.registers[RegisterID::RPC as usize].value += offset - 1; // forward jump, minus one since the intepreter will automatically add 1 after instruction
        }
    }

    #[inline(always)]
    fn load_constant_to_register(&mut self, instruction: Instruction) {
        let destination_register = InstructionDecoder::decode_destination_register(instruction);
        let immutable_address = InstructionDecoder::decode_immutable_address_small(instruction);

        let register = Register {
            kind: RegisterValueKind::ImmAddress,
            value: immutable_address,
        };
        self.set_value_in_register(destination_register, register);
    }

    #[inline(always)]
    fn load_float32_to_register(&mut self, instruction: Instruction) {
        let destination_register = InstructionDecoder::decode_destination_register(instruction);

        let number = self.get_next_instruction();
        let number = f32::from_bits(number);
        self.load_number_to_register(destination_register, number);
    }

    #[inline(always)]
    fn load_nil_to_register(&mut self, instruction: Instruction) {
        let destination = InstructionDecoder::decode_destination_register(instruction);
        let register = Register::new(RegisterValueKind::None, 0);
        self.set_value_in_register(destination, register);
    }

    #[inline(always)]
    fn load_memory_address_to_register(&mut self, destination: Instruction, address: Instruction) {
        let value = Register::new(RegisterValueKind::MemAddress, address);
        self.set_value_in_register(destination, value);
    }

    #[inline(always)]
    fn load_number_to_register(&mut self, destination: Instruction, number: f32) {
        let number = number.to_bits();
        let register = Register::new(RegisterValueKind::Float32, number);
        self.set_value_in_register(destination, register);
    }

    #[inline(always)]
    fn load_bool_to_register(&mut self, instruction: Instruction) {
        let destination = InstructionDecoder::decode_destination_register(instruction);
        let boolean = InstructionDecoder::decode_immutable_address_small(instruction);
        let register = Register::new(RegisterValueKind::Float32, boolean);
        self.set_value_in_register(destination, register);
    }

    #[inline(always)]
    fn emit_error_with_message(&mut self, message: &str) {
        let address = self.load_object_to_memory(NovaObject::String(Box::new(message.to_string())));
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

    #[inline(always)]
    fn load_object_to_memory(&mut self, object: NovaObject) -> Instruction {
        self.memory.push(object);
        let address = self.memory.len() - 1;
        address as Instruction
    }

    #[inline(always)]
    fn get_next_instruction(&mut self) -> Instruction {
        let instruction = self.peek_next_instruction();
        self.registers[RegisterID::RPC as usize].value += 1;
        instruction
    }

    #[inline(always)]
    fn get_register(&self, register_id: Instruction) -> Register {
        self.registers[register_id as usize]
    }

    #[inline(always)]
    fn set_value_in_register(&mut self, register_id: Instruction, value: Register) {
        self.registers[register_id as usize] = value
    }

    #[inline(always)]
    fn get_object_from_memory(&self, address: Instruction) -> &NovaObject {
        &self.memory[address as usize]
    }

    #[inline(always)]
    fn free_memory_location(&mut self, _address: Instruction) {
        todo!()
    }
    #[inline(always)]
    fn define_global_indirect(&mut self, instruction: Instruction) {
        let index = InstructionDecoder::decode_immutable_address_small(instruction);
        let immutable = self.immutables[index as usize].clone();

        if let NovaObject::String(name) = immutable {
            let global_location = self.allocate_global();
            self.identifiers.insert(name.to_string(), global_location);
        }
    }

    #[inline(always)]
    fn allocate_global(&mut self) -> Instruction {
        self.globals.push(Register::default());
        return (self.globals.len() - 1) as Instruction;
    }

    #[inline(always)]
    fn set_global_value(&mut self, address: Instruction, new_value: Register) {
        let current_value = self.globals[address as usize];

        if current_value.kind.is_mem_address() {
            self.free_memory_location(current_value.value);
        }

        self.globals[address as usize] = new_value;
    }

    #[inline(always)]
    fn load_global_value(&mut self, destination: Instruction, global_address: Instruction) {
        let value = self.globals[global_address as usize];
        self.registers[destination as usize] = value;
    }

    #[inline(always)]
    fn store_global_indirect(&mut self, instruction: Instruction) {
        let source = InstructionDecoder::decode_source_register_1(instruction);
        let index = InstructionDecoder::decode_immutable_address_small(instruction);

        let immutable = self.immutables[index as usize].clone();

        if let NovaObject::String(name) = immutable {
            let global_address = self.identifiers.get(name.as_str());

            if let Some(&address) = global_address {
                let register = self.get_register(source);
                self.set_global_value(address, register);

                return;
            }

            self.emit_error_with_message(&format!("Cannot find global named: {}", name));
            return;
        }

        self.emit_error_with_message(&format!("Invalid global identifier: {:?}", immutable));
    }

    #[inline(always)]
    fn load_global_indirect(&mut self, instruction: Instruction) {
        let destination = InstructionDecoder::decode_destination_register(instruction);
        let index = InstructionDecoder::decode_immutable_address_small(instruction);

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
        let number = InstructionDecoder::decode_immutable_address_small(instruction);
        let mut local_space = vec![Register::default(); number as usize];

        self.locals.append(&mut local_space)
    }

    #[inline(always)]
    fn deallocate_locals(&mut self, instruction: Instruction) {
        // number of variables
        let mut number = InstructionDecoder::decode_immutable_address_small(instruction);
        
        while number > 0 {
            number -= 1;
            self.locals.pop();
        }

    }

    #[inline(always)]
    fn store_local(&mut self, instruction: Instruction) {
        let source = InstructionDecoder::decode_source_register_1(instruction);
        let address = InstructionDecoder::decode_immutable_address_small(instruction);

        let register = self.get_register(source);
        let local_offset = self.registers[RegisterID::RLO as usize].value;
        self.locals[(address + local_offset) as usize] = register;
    }

    #[inline(always)]
    fn load_local(&mut self, instruction: Instruction) {
        let destination = InstructionDecoder::decode_destination_register(instruction);
        let address = InstructionDecoder::decode_immutable_address_small(instruction);

        let local_offset = self.registers[RegisterID::RLO as usize].value;
        let register = self.locals[(address + local_offset) as usize];
        self.set_value_in_register(destination, register);
    }

    #[inline(always)]
    fn compare_registers(&mut self, op: OpCode, first: Register, second: Register) -> bool {
        match op {
            OpCode::LessJump => {
                if first.kind.is_none() && second.kind.is_float32() {
                    return true;
                }

                if (first.kind.is_none() || first.kind.is_float32()) && second.kind.is_mem_address()
                {
                    return true;
                }

                if (first.kind.is_none() || first.kind.is_float32()) && second.kind.is_imm_address()
                {
                    return true;
                }

                if first.kind.is_float32() && second.kind.is_float32() {
                    let first = f32::from_bits(first.value);
                    let second = f32::from_bits(second.value);
                    return first < second;
                }

                if first.kind.is_mem_address() && second.kind.is_mem_address() {
                    let first = self.get_object_from_memory(first.value);
                    let second = self.get_object_from_memory(second.value);

                    return first < second;
                }

                if first.kind.is_imm_address() && second.kind.is_imm_address() {
                    let first = &self.immutables[first.value as usize];
                    let second = &self.immutables[second.value as usize];

                    return first < second;
                }

                if first.kind.is_imm_address() && second.kind.is_mem_address() {
                    let first = &self.immutables[first.value as usize];
                    let second = self.get_object_from_memory(second.value);

                    return first < second;
                }

                if first.kind.is_mem_address() && second.kind.is_imm_address() {
                    let first = self.get_object_from_memory(first.value);
                    let second = &self.immutables[second.value as usize];

                    return first < second;
                }

                self.emit_error_with_message(&format!(
                    "cannot compare {:?} to {:?}",
                    first.kind, second.kind
                ));
            }

            OpCode::LessEqualJump => {
                if first.kind.is_none() && second.kind.is_float32() {
                    return true;
                }

                if (first.kind.is_none() || first.kind.is_float32()) && second.kind.is_mem_address()
                {
                    return true;
                }

                if first.kind.is_float32() && second.kind.is_float32() {
                    let first = f32::from_bits(first.value);
                    let second = f32::from_bits(second.value);
                    return first <= second;
                }

                if first.kind.is_mem_address() && second.kind.is_mem_address() {
                    let first = self.get_object_from_memory(first.value);
                    let second = self.get_object_from_memory(second.value);

                    return first <= second;
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
        self.instructions[self.registers[RegisterID::RPC as usize].value as usize]
    }

    #[cfg(feature = "debug")]
    fn debug(&self) {
        #[cfg(feature = "dbg_code")]
        debug_instruction(
            &self.instructions,
            self.registers[RegisterID::RPC as usize].value,
        );
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
fn print_vec_of_registers(registers: &Vec<Register>) {
    println!("[");
    for (index, register) in registers.iter().enumerate() {
        println!("\t[{}] {}, ",index, register)
    }
    println!("]");
}