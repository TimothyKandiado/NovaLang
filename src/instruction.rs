use crate::bytecode::OpCode;

pub type Instruction = u32;

pub struct InstructionBuilder {
    instruction: Instruction,
}

impl Default for InstructionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl InstructionBuilder {
    pub fn new() -> Self {
        Self { instruction: 0 }
    }

    pub fn build(self) -> Instruction {
        self.instruction
    }

    pub fn new_binary_op_instruction(
        op: OpCode,
        destination: Instruction,
        source1: Instruction,
        source2: Instruction,
    ) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(op)
            .add_destination_register(destination)
            .add_source_register_1(source1)
            .add_source_register_2(source2)
            .build()
    }

    pub fn new_comparison_instruction(
        op: OpCode,
        source1: Instruction,
        source2: Instruction,
    ) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(op)
            .add_source_register_1(source1)
            .add_source_register_2(source2)
            .build()
    }

    pub fn new_jump_instruction(offset: Instruction, forward: bool) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::Jump)
            .add_destination_register(forward as Instruction)
            .add_address_small(offset)
            .build()
    }

    pub fn new_load_constant_instruction(
        destination: Instruction,
        constant_index: Instruction,
    ) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::LoadK)
            .add_destination_register(destination)
            .add_address_small(constant_index)
            .build()
    }

    pub fn new_load_float32_instruction(
        destination: Instruction,
    ) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::LoadFloat)
            .add_destination_register(destination)
            .build()
    }

    pub fn new_move_instruction(destination: Instruction, source: Instruction) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::Move)
            .add_destination_register(destination)
            .add_source_register_1(source)
            .build()
    }

    pub fn new_print_instruction(source: Instruction, newline: bool) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::Print)
            .add_destination_register(newline as Instruction)
            .add_source_register_1(source)
            .build()
    }

    pub fn new_halt_instruction() -> Instruction {
        InstructionBuilder::new().add_opcode(OpCode::Halt).build()
    }

    pub fn add_opcode(mut self, opcode: OpCode) -> Self {
        let opcode = opcode as Instruction;
        let shifted = opcode << 26;
        self.instruction += shifted;
        self
    }

    pub fn add_destination_register(mut self, destination: Instruction) -> Self {
        let shifted = destination << 22;
        self.instruction += shifted;
        self
    }

    pub fn add_source_register_1(mut self, source: Instruction) -> Self {
        let shifted = source << 18;
        self.instruction += shifted;
        self
    }

    pub fn add_source_register_2(mut self, source: Instruction) -> Self {
        self.instruction += source;
        self
    }

    pub fn add_address_small(mut self, address: Instruction) -> Self {
        self.instruction += address;
        self
    }
}

pub struct InstructionDecoder {}
impl InstructionDecoder {
    #[inline(always)]
    pub fn decode_opcode(instruction: Instruction) -> Instruction {
        
        instruction >> 26
    }

    #[inline(always)]
    pub fn decode_destination_register(instruction: Instruction) -> Instruction {
        let instruction = instruction >> 22;
        
        instruction & 0xF
    }

    #[inline(always)]
    pub fn decode_source_register_1(instruction: Instruction) -> Instruction {
        let instruction = instruction >> 18;
        
        instruction & 0xF
    }

    #[inline(always)]
    pub fn decode_source_register_2(instruction: Instruction) -> Instruction {
        
        instruction & 0xF
    }

    #[inline(always)]
    pub fn decode_immutable_address_small(instruction: Instruction) -> Instruction {
        
        instruction & 0x7FFF
    }

    #[inline(always)]
    pub fn decode_float32(instruction: Instruction) -> f32 {
        f32::from_bits(instruction)
    }
}

#[cfg(test)]
mod instruction_builder_tests {
    use super::{Instruction, InstructionBuilder, InstructionDecoder};
    use crate::bytecode::OpCode;

    #[test]
    fn test_opcode_encoding_and_decoding() {
        // prefix r_ means raw, d_ means decoded
        let code = OpCode::Break;
        let r_code = code as Instruction;
        let instruction = InstructionBuilder::new().add_opcode(code).build();
        let d_code = InstructionDecoder::decode_opcode(instruction);

        assert_eq!(r_code, d_code);
    }

    #[test]
    fn test_register_encoding_and_decoding() {
        let r_destination = 5u32;
        let r_source1 = 3u32;
        let r_source2 = 4u32;

        let instruction = InstructionBuilder::new()
            .add_destination_register(r_destination)
            .add_source_register_1(r_source1)
            .add_source_register_2(r_source2)
            .build();

        let d_destination = InstructionDecoder::decode_destination_register(instruction);
        let d_source1 = InstructionDecoder::decode_source_register_1(instruction);
        let d_source2 = InstructionDecoder::decode_source_register_2(instruction);

        assert_eq!(r_destination, d_destination);
        assert_eq!(r_source1, d_source1);
        assert_eq!(r_source2, d_source2);
    }

    #[test]
    fn test_immutable_encoding_and_decoding() {
        let r_immutable = 20u32;

        let instruction = InstructionBuilder::new()
            .add_address_small(r_immutable)
            .build();

        let d_immutable = InstructionDecoder::decode_immutable_address_small(instruction);

        assert_eq!(r_immutable, d_immutable);
    }

    #[test]
    fn test_decode_float32() {
        let r_number = 50.0f32;
        let bits = r_number.to_bits();
        let instruction = bits;
        let d_number = InstructionDecoder::decode_float32(instruction);

        assert_eq!(r_number, d_number)
    }
}
