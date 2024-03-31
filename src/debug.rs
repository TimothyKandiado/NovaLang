use crate::{
    bytecode::OpCode,
    instruction::{Instruction, InstructionDecoder},
};

pub fn debug_instruction(instructions: &Vec<Instruction>, instruction_pointer: Instruction) {
    let instruction = instructions[instruction_pointer as usize];

    let opcode = InstructionDecoder::decode_opcode(instruction);

    print!("(dbg[{}]) ", instruction_pointer);
    match opcode {
        // System Interrupt
        x if x == OpCode::Halt as u32 => {
            println!("HALT");
        }

        // Binary Operations
        x if x == OpCode::Add as u32 => {
            binary_op("ADD", instruction);
        }
        x if x == OpCode::Sub as u32 => {
            binary_op("SUB", instruction);
        }
        x if x == OpCode::Mul as u32 => {
            binary_op("MUL", instruction);
        }
        x if x == OpCode::Div as u32 => {
            binary_op("DIV", instruction);
        }
        x if x == OpCode::Pow as u32 => {
            binary_op("POW", instruction);
        }
        x if x == OpCode::Mod as u32 => {
            binary_op("MOD", instruction);
        }

        // Register Manipulation
        x if x == OpCode::LoadK as u32 => {
            load_constant_to_register(instruction);
        }

        x if x == OpCode::LoadBool as u32 => {
            load_bool_to_register(instruction);
        }

        x if x == OpCode::LoadFloat as u32 => {
            let destination_register = InstructionDecoder::decode_destination_register(instruction);
            let number = instructions[instruction_pointer as usize + 1];
            let number = f32::from_bits(number);
            load_number_to_register(destination_register, number);
        }

        x if x == OpCode::Move as u32 => {
            move_register(instruction);
        }

        // Variable Manipulation
        x if x == OpCode::DefineGlobalIndirect as u32 => {
            let address = InstructionDecoder::decode_immutable_address_small(instruction);

            println!("DEFINEGLOBALINDIRECT {}", address);
        }

        x if x == OpCode::StoreGlobalIndirect as u32 => {
            let source1 = InstructionDecoder::decode_source_register_1(instruction);
            let address = InstructionDecoder::decode_immutable_address_small(instruction);

            println!("STOREGLOBALINDIRECT {} {}", source1, address);
        }

        x if x == OpCode::LoadGlobalIndirect as u32 => {
            let destination = InstructionDecoder::decode_destination_register(instruction);
            let address = InstructionDecoder::decode_immutable_address_small(instruction);

            println!("LOADGLOBALINDIRECT {} {}", destination, address);
        }
        

        x if x == OpCode::LoadGlobal as u32 => {
            let destination = InstructionDecoder::decode_destination_register(instruction);
            let address = InstructionDecoder::decode_immutable_address_small(instruction);

            println!("LOADGLOBAL {} {}", destination, address);
        }

        x if x == OpCode::AllocateLocal as u32 => {
            let number = InstructionDecoder::decode_immutable_address_small(instruction);

            println!("ALLOCATELOCAL {}", number);
        }

        x if x == OpCode::DeallocateLocal as u32 => {
            let number = InstructionDecoder::decode_immutable_address_small(instruction);
            println!("DEALLOCATELOCAL {}", number);
        }

        x if x == OpCode::StoreLocal as u32 => {
            let source1 = InstructionDecoder::decode_source_register_1(instruction);
            let address = InstructionDecoder::decode_immutable_address_small(instruction);

            println!("STORELOCAL {} {}", source1, address);
        }

        x if x == OpCode::LoadLocal as u32 => {
            let destination = InstructionDecoder::decode_destination_register(instruction);
            let address = InstructionDecoder::decode_immutable_address_small(instruction);

            println!("LOADLOCAL {} {}", destination, address);
        }

        // Control flow
        x if x == OpCode::LessJump as u32 => {
            let source1 = InstructionDecoder::decode_source_register_1(instruction);
            let source2 = InstructionDecoder::decode_source_register_2(instruction);

            println!("LESSJ {} {}", source1, source2);
        }

        x if x == OpCode::Jump as u32 => {
            let offset = InstructionDecoder::decode_immutable_address_small(instruction);
            let direction = InstructionDecoder::decode_destination_register(instruction);

            println!(
                "JUMP {} {}",
                offset,
                if direction == 0 { "back" } else { "forward" }
            );
        }

        x if x == OpCode::NewFrame as u32 => {
            println!("NEWFRAME");
        }

        x if x == OpCode::Return as u32 => {
            println!("RETURN");
        }

        // IO
        x if x == OpCode::Print as u32 => {
            let source = InstructionDecoder::decode_source_register_1(instruction);
            println!("PRINT {}", source);
        }

        _ => println!("Unsupported opcode instruction ({:#x})", opcode),
    }
}

fn binary_op(name: &str, instruction: Instruction) {
    let destination_register = InstructionDecoder::decode_destination_register(instruction);
    let source_register_1 = InstructionDecoder::decode_source_register_1(instruction);
    let source_register_2 = InstructionDecoder::decode_source_register_2(instruction);

    println!(
        "{} {} {} {}",
        name, destination_register, source_register_1, source_register_2
    );
}

fn load_constant_to_register(instruction: Instruction) {
    let destination_register = InstructionDecoder::decode_destination_register(instruction);
    let immutable_address = InstructionDecoder::decode_immutable_address_small(instruction);

    println!("LOADK {} {}", destination_register, immutable_address);
}

fn load_bool_to_register(instruction: Instruction) {
    let destination = InstructionDecoder::decode_destination_register(instruction);
    let boolean = InstructionDecoder::decode_immutable_address_small(instruction);

    println!(
        "LOADBOOL {} {}",
        destination,
        if boolean == 0 { "false" } else { "true" }
    );
}

fn load_number_to_register(destination: Instruction, number: f32) {
    println!("LOADFLOAT {} {}", destination, number);
}

fn move_register(instruction: Instruction) {
    let destination = InstructionDecoder::decode_destination_register(instruction);
    let source = InstructionDecoder::decode_source_register_1(instruction);

    println!("MOVE {} {}", destination, source);
}
