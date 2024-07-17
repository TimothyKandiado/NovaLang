use crate::{
    bytecode::OpCode,
    instruction::{Instruction, InstructionDecoder},
};

pub fn debug_instruction(
    instructions: &[Instruction],
    instruction_pointer: u64,
) -> String {
    let instruction = instructions[instruction_pointer as usize];

    let opcode = InstructionDecoder::decode_opcode(instruction);

    //print!("(dbg[{}]) ", instruction_pointer);
    match opcode {
        x if x == OpCode::NoInstruction as u32 => {
            "NOINSTRUCTION".to_string()
        }
        // System Interrupt
        x if x == OpCode::Halt as u32 => {
            "HALT".to_string()
        }

        // Binary Operations
        x if x == OpCode::Add as u32 => binary_op("ADD", instruction),
        x if x == OpCode::Sub as u32 => binary_op("SUB", instruction),
        x if x == OpCode::Mul as u32 => binary_op("MUL", instruction),
        x if x == OpCode::Div as u32 => binary_op("DIV", instruction),
        x if x == OpCode::Pow as u32 => binary_op("POW", instruction),
        x if x == OpCode::Mod as u32 => binary_op("MOD", instruction),

        // Register Manipulation
        x if x == OpCode::LoadK as u32 => load_constant_to_register(instruction),

        x if x == OpCode::LoadBool as u32 => load_bool_to_register(instruction),

        x if x == OpCode::LoadFloat32 as u32 => {
            let destination_register = InstructionDecoder::decode_destination_register(instruction);
            let number = instructions[instruction_pointer as usize + 1];
            let number = f32::from_bits(number);
            load_float32_to_register(destination_register, number)
        }

        x if x == OpCode::LoadFloat64 as u32 => {
            let destination_register = InstructionDecoder::decode_destination_register(instruction);
            let first_half = instructions[instruction_pointer as usize + 1];
            let second_half = instructions[instruction_pointer as usize + 2];
            let number = InstructionDecoder::merge_u32s(first_half, second_half);
            let number = f64::from_bits(number);
            load_float64_to_register(destination_register, number)
        }

        x if x == OpCode::LoadInt32 as u32 => {
            let destination_register = InstructionDecoder::decode_destination_register(instruction);
            let number = instructions[instruction_pointer as usize + 1];
            let number = number as i32;
            format!("LOADINT32 {} {}", destination_register, number)
        }

        x if x == OpCode::LoadInt64 as u32 => {
            let destination_register = InstructionDecoder::decode_destination_register(instruction);
            let first_half = instructions[instruction_pointer as usize + 1];
            let second_half = instructions[instruction_pointer as usize + 2];
            let number = InstructionDecoder::merge_u32s(first_half, second_half);
            let number = number as i64;
            format!("LOADINT64 {} {}", destination_register, number)
        }

        x if x == OpCode::Move as u32 => move_register(instruction),

        // Variable Manipulation
        x if x == OpCode::DefineGlobalIndirect as u32 => {
            let address = InstructionDecoder::decode_immutable_address_small(instruction);

            format!("DEFINEGLOBALINDIRECT {}", address)
        }

        x if x == OpCode::StoreGlobalIndirect as u32 => {
            let source1 = InstructionDecoder::decode_source_register_1(instruction);
            let address = InstructionDecoder::decode_immutable_address_small(instruction);

            format!("STOREGLOBALINDIRECT {} {}", source1, address)
        }

        x if x == OpCode::LoadGlobalIndirect as u32 => {
            let destination = InstructionDecoder::decode_destination_register(instruction);
            let address = InstructionDecoder::decode_immutable_address_small(instruction);

            format!("LOADGLOBALINDIRECT {} {}", destination, address)
        }

        x if x == OpCode::LoadGlobal as u32 => {
            let destination = InstructionDecoder::decode_destination_register(instruction);
            let address = InstructionDecoder::decode_immutable_address_small(instruction);

            format!("LOADGLOBAL {} {}", destination, address)
        }

        x if x == OpCode::AllocateLocal as u32 => {
            let number = InstructionDecoder::decode_immutable_address_small(instruction);

            format!("ALLOCATELOCAL {}", number)
        }

        x if x == OpCode::DeallocateLocal as u32 => {
            let number = InstructionDecoder::decode_immutable_address_small(instruction);
            format!("DEALLOCATELOCAL {}", number)
        }

        x if x == OpCode::StoreLocal as u32 => {
            let source1 = InstructionDecoder::decode_source_register_1(instruction);
            let address = InstructionDecoder::decode_immutable_address_small(instruction);

            format!("STORELOCAL {} {}", source1, address)
        }

        x if x == OpCode::LoadLocal as u32 => {
            let destination = InstructionDecoder::decode_destination_register(instruction);
            let address = InstructionDecoder::decode_immutable_address_small(instruction);

            format!("LOADLOCAL {} {}", destination, address)
        }

        // Control flow
        x if x == OpCode::Less as u32 => {
            let source1 = InstructionDecoder::decode_source_register_1(instruction);
            let source2 = InstructionDecoder::decode_source_register_2(instruction);

            format!("LESS {} {}", source1, source2)
        }

        x if x == OpCode::LessEqual as u32 => {
            let source1 = InstructionDecoder::decode_source_register_1(instruction);
            let source2 = InstructionDecoder::decode_source_register_2(instruction);

            format!("LESSEQUAL {} {}", source1, source2)
        }

        x if x == OpCode::Equal as u32 => {
            let source1 = InstructionDecoder::decode_source_register_1(instruction);
            let source2 = InstructionDecoder::decode_source_register_2(instruction);

            format!("EQUAL {} {}", source1, source2)
        }

        x if x == OpCode::JumpFalse as u32 => {
            let source1 = InstructionDecoder::decode_source_register_1(instruction);

            format!("JUMPFALSE {}", source1)
        }

        x if x == OpCode::Jump as u32 => {
            let offset = InstructionDecoder::decode_immutable_address_small(instruction);
            let direction = InstructionDecoder::decode_destination_register(instruction);

            format!(
                "JUMP {} {}",
                offset,
                if direction == 0 { "back" } else { "forward" }
            )
        }

        x if x == OpCode::NewFrame as u32 => {
            "NEWFRAME".to_string()
        }

        x if x == OpCode::CallIndirect as u32 => {
            let parameter_start = InstructionDecoder::decode_destination_register(instruction);
            let parameters = InstructionDecoder::decode_source_register_1(instruction);
            let name_address = InstructionDecoder::decode_immutable_address_small(instruction);

            format!(
                "CALL_INDIRECT {} {} {} ",
                parameter_start, parameters, name_address
            )
        }

        x if x == OpCode::Invoke as u32 => {
            let parameter_start = InstructionDecoder::decode_destination_register(instruction);
            let parameters = InstructionDecoder::decode_source_register_1(instruction);
            let invoke_register = InstructionDecoder::decode_source_register_2(instruction);

            format!(
                "INVOKE {} {} {} ",
                parameter_start, parameters, invoke_register
            )
        }

        x if x == OpCode::ReturnNone as u32 => {
            "RETURN_NONE".to_string()
        }

        x if x == OpCode::ReturnVal as u32 => {
            let source = InstructionDecoder::decode_source_register_1(instruction);
            format!("RETURN_VAL {}", source)
        }

        x if x == OpCode::LoadReturn as u32 => {
            let destination = InstructionDecoder::decode_destination_register(instruction);
            format!("LOADRETURN {}", destination)
        }

        // IO
        x if x == OpCode::Print as u32 => {
            let source = InstructionDecoder::decode_source_register_1(instruction);
            format!("PRINT {}", source)
        }

        // Logical
        x if x == OpCode::And as u32 => {
            let source1 = InstructionDecoder::decode_source_register_1(instruction);
            let source2 = InstructionDecoder::decode_source_register_2(instruction);

            format!("AND {} {}", source1, source2)
        }

        x if x == OpCode::Not as u32 => {
            let source1 = InstructionDecoder::decode_source_register_1(instruction);

            format!("NOT {}", source1)
        }

        x if x == OpCode::Neg as u32 => {
            let source1 = InstructionDecoder::decode_source_register_1(instruction);

            format!("NEGATE {}", source1)
        }

        _ => format!("Unsupported opcode instruction ({:#x})", opcode),
    }
}

fn binary_op(name: &str, instruction: Instruction) -> String {
    let destination_register = InstructionDecoder::decode_destination_register(instruction);
    let source_register_1 = InstructionDecoder::decode_source_register_1(instruction);
    let source_register_2 = InstructionDecoder::decode_source_register_2(instruction);

    format!(
        "{} {} {} {}",
        name, destination_register, source_register_1, source_register_2
    )
}

fn load_constant_to_register(instruction: Instruction) -> String {
    let destination_register = InstructionDecoder::decode_destination_register(instruction);
    let immutable_address = InstructionDecoder::decode_immutable_address_small(instruction);

    format!("LOADK {} {}", destination_register, immutable_address)
}

fn load_bool_to_register(instruction: Instruction) -> String {
    let destination = InstructionDecoder::decode_destination_register(instruction);
    let boolean = InstructionDecoder::decode_immutable_address_small(instruction);

    format!(
        "LOADBOOL {} {}",
        destination,
        if boolean == 0 { "false" } else { "true" }
    )
}

fn load_float32_to_register(destination: Instruction, number: f32) -> String {
    format!("LOADFLOAT32 {} {}", destination, number)
}

fn load_float64_to_register(destination: Instruction, number: f64) -> String {
    format!("LOADFLOAT64 {} {}", destination, number)
}

fn move_register(instruction: Instruction) -> String {
    let destination = InstructionDecoder::decode_destination_register(instruction);
    let source = InstructionDecoder::decode_source_register_1(instruction);

    format!("MOVE {} {}", destination, source)
}
