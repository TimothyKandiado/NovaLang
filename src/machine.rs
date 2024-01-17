use crate::{
    bytecode::OpCode,
    instruction::{Instruction, InstructionDecoder},
    object::{NovaObject, ObjectKind},
    program::Program,
    register::{Register, RegisterID},
};

const PC_START: Instruction = 0x0;

pub struct VirtualMachine {
    instructions: Vec<Instruction>,
    immutables: Vec<NovaObject>,
    registers: [Register; RegisterID::RMax as usize + 1],
    running: bool,
    instruction_count: usize,
    memory: Vec<NovaObject>,
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
            let instruction = self.get_next_instruction();

            self.execute_instruction(instruction);

            if self.check_error() {
                break;
            }
            self.debug();
        }
        
        let result = self.get_value_from_register(RegisterID::R0 as u32);
        println!("result = {}", result)
    }

    fn execute_instruction(&mut self, instruction: Instruction) {
        let opcode = InstructionDecoder::decode_opcode(instruction);

        match opcode {
            x if x == OpCode::Halt as u32 => {
                self.running = false;
            }
            x if x == OpCode::Add as u32 => {
                self.add(instruction);
            }

            x if x == OpCode::LoadK as u32 => {
                self.load_constant_to_register(instruction);
            }

            _ => self.emit_error_with_message("Undefined opcode instruction"),
        }
    }

    #[inline(always)]
    fn add(&mut self, instruction: Instruction) {
        let destination_register = InstructionDecoder::decode_destination_register(instruction);
        let source_register_1 = InstructionDecoder::decode_source_register_1(instruction);
        let source_register_2 = InstructionDecoder::decode_source_register_2(instruction);

        let register_1 = self.get_value_from_register(source_register_1);
        let register_2 = self.get_value_from_register(source_register_2);

        if let (ObjectKind::Float32, ObjectKind::Float32) = (register_1.kind, register_2.kind) {
            let value_1 = f32::from_bits(register_1.value);
            let value_2 = f32::from_bits(register_2.value);

            let sum = value_1 + value_2;
            let sum = sum.to_bits();

            let new_value = Register::new(register_1.kind, sum);
            self.set_value_in_register(destination_register, new_value);
            return;
        }

        self.emit_error_with_message(&format!("cannot add {:?} to {:?}", register_1.kind, register_2.kind))
    }

    #[inline(always)]
    fn load_constant_to_register(&mut self, instruction: Instruction) {
        let destination_register = InstructionDecoder::decode_destination_register(instruction);
        let immutable_address = InstructionDecoder::decode_immutable_address_small(instruction);

        let immutable_object = &self.immutables[immutable_address as usize];

        match immutable_object {
            NovaObject::None => self.load_nil_to_register(destination_register),
            NovaObject::Number(number) => {
                self.load_number_to_register(destination_register, *number)
            }
            NovaObject::Bool(bool) => self.load_bool_to_register(destination_register, *bool),

            _ => {
                let address = self.load_object_to_memory(immutable_object.clone());
                self.load_memory_address_to_register(destination_register, address);
            }
        }
    }

    #[inline(always)]
    fn load_nil_to_register(&mut self, destination: Instruction) {
        let register = Register::new(ObjectKind::None, 0);
        self.set_value_in_register(destination, register);
    }

    #[inline(always)]
    fn load_memory_address_to_register(&mut self, destination: Instruction, address: Instruction) {
        let value = Register::new(ObjectKind::MemAddress, address);
        self.set_value_in_register(destination, value);
    }

    #[inline(always)]
    fn load_number_to_register(&mut self, destination: Instruction, number: f32) {
        let number = number.to_bits();
        let register = Register::new(ObjectKind::Float32, number);
        self.set_value_in_register(destination, register);
    }

    #[inline(always)]
    fn load_bool_to_register(&mut self, destination: Instruction, boolean: bool) {
        let register = Register::new(ObjectKind::Float32, boolean as u32);
        self.set_value_in_register(destination, register);
    }

    #[inline(always)]
    fn emit_error_with_message(&mut self, message: &str) {
        let address = self.load_object_to_memory(NovaObject::String(message.to_string()));
        self.load_memory_address_to_register(RegisterID::RERR as Instruction, address);
        
        //println!("Error: {}", message)
    }

    #[inline(always)]
    fn check_error(&self) -> bool {
        let register = self.get_value_from_register(RegisterID::RERR as Instruction);

        if let ObjectKind::MemAddress = register.kind {
            let address = register.value;
            let object = &self.memory[address as usize];
            print!("Error Occurred: ");
            
            if let NovaObject::String(string) = object {
                print!("{}", string)
            }
            println!()
        }

        return false;
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
    fn get_value_from_register(&self, register_id: Instruction) -> Register {
        self.registers[register_id as usize]
    }

    #[inline(always)]
    fn set_value_in_register(&mut self, register_id: Instruction, value: Register) {
        self.registers[register_id as usize] = value
    }

    #[inline(always)]
    fn peek_next_instruction(&self) -> Instruction {
        let instruction =
            self.instructions[self.registers[RegisterID::RPC as usize].value as usize];
        instruction
    }

    fn debug(&self) {
        self.print_register_values()
    }

    fn print_register_values(&self) {
        println!("{:=^30}", "Registers");
        for register_index in 0..RegisterID::RMax as usize + 1 {
            let register = self.get_value_from_register(register_index as u32);
            println!("R{:<2}: {}", register_index, register);
        }
        println!("{:=^30}", "");
    }
}
