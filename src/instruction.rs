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

    pub fn from(instruction: Instruction) -> InstructionBuilder {
        let mut instruction_builder = InstructionBuilder::new();
        instruction_builder.instruction = instruction;
        instruction_builder
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

    pub fn new_not_instruction(source1: Instruction) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::Not)
            .add_destination_register(source1)
            .add_source_register_1(source1)
            .build()
    }

    pub fn new_jump_instruction(offset: Instruction, forward: bool) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::Jump)
            .add_destination_register(forward as Instruction)
            .add_address_small(offset)
            .build()
    }

    pub fn new_jump_false_instruction(source1: Instruction) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::JumpFalse)
            .add_source_register_1(source1)
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

    pub fn new_invoke_instruction(
        parameter_start: Instruction,
        parameter_number: Instruction,
        register_index: Instruction,
    ) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::Invoke)
            .add_destination_register(parameter_start)
            .add_source_register_1(parameter_number)
            .add_source_register_2(register_index)
            .build()
    }

    pub fn new_load_bool(destination: Instruction, value: Instruction) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::LoadBool)
            .add_destination_register(destination)
            .add_address_small(value)
            .build()
    }

    pub fn new_define_global_indirect(immutable_address: Instruction) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::DefineGlobalIndirect)
            .add_address_small(immutable_address)
            .build()
    }

    pub fn new_store_global_indirect(
        source1: Instruction,
        immutable_address: Instruction,
    ) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::StoreGlobalIndirect)
            .add_source_register_1(source1)
            .add_address_small(immutable_address)
            .build()
    }

    pub fn new_load_global_indirect(destination: Instruction, address: Instruction) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::LoadGlobalIndirect)
            .add_destination_register(destination)
            .add_address_small(address)
            .build()
    }

    pub fn new_allocate_local(number: Instruction) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::AllocateLocal)
            .add_address_small(number)
            .build()
    }

    pub fn new_deallocate_local(number: Instruction) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::DeallocateLocal)
            .add_address_small(number)
            .build()
    }

    pub fn new_store_local(source1: Instruction, destination_variable: Instruction) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::StoreLocal)
            .add_source_register_1(source1)
            .add_address_small(destination_variable)
            .build()
    }

    pub fn new_load_local(destination: Instruction, source_variable: Instruction) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::LoadLocal)
            .add_destination_register(destination)
            .add_address_small(source_variable)
            .build()
    }

    pub fn new_load_float32_instruction(destination: Instruction) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::LoadFloat32)
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

    pub fn new_return_none_instruction() -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::ReturnNone)
            .build()
    }

    pub fn new_return_value(source: Instruction) -> Instruction {
        InstructionBuilder::new()
            .add_opcode(OpCode::ReturnVal)
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

    pub fn clear_address_small(mut self) -> Self {
        self.instruction >>= 16;
        self.instruction <<= 16;
        self
    }
}

pub mod instruction_decoder {
    use super::Instruction;

    #[inline(always)]
    pub fn split_u64(value: u64) -> (Instruction, Instruction) {
        let first_half = (value >> 32) as u32;
        let last_half = value as u32;

        (first_half, last_half)
    }

    #[inline(always)]
    pub fn merge_u32s(first_half: u32, second_half: u32) -> u64 {
        let first_half = first_half as u64;
        let second_half = second_half as u64;

        let first_half = first_half << 32;

        first_half + second_half
    }

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
        // take only lower 4 bits
        instruction & 0xF
    }

    #[inline(always)]
    pub fn decode_immutable_address_small(instruction: Instruction) -> Instruction {
        // take only lower 16 bits
        instruction & 0xffff
    }

    #[inline(always)]
    pub fn decode_float32(instruction: Instruction) -> f32 {
        f32::from_bits(instruction)
    }
}

#[cfg(test)]
mod instruction_builder_tests {
    use super::{instruction_decoder, Instruction, InstructionBuilder};
    use crate::bytecode::OpCode;

    #[test]
    fn test_opcode_encoding_and_decoding() {
        // prefix r_ means raw, d_ means decoded
        let code = OpCode::Break;
        let r_code = code as Instruction;
        let instruction = InstructionBuilder::new().add_opcode(code).build();
        let d_code = instruction_decoder::decode_opcode(instruction);

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

        let d_destination = instruction_decoder::decode_destination_register(instruction);
        let d_source1 = instruction_decoder::decode_source_register_1(instruction);
        let d_source2 = instruction_decoder::decode_source_register_2(instruction);

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

        let d_immutable = instruction_decoder::decode_immutable_address_small(instruction);

        assert_eq!(r_immutable, d_immutable);
    }

    #[test]
    fn test_decode_float32() {
        let r_number = 50.0f32;
        let bits = r_number.to_bits();
        let instruction = bits;
        let d_number = instruction_decoder::decode_float32(instruction);

        assert_eq!(r_number, d_number)
    }

    #[test]
    fn test_splitting_and_merging_u64() {
        let number = 100000000000u64;

        let (first, second) = instruction_decoder::split_u64(number);
        let merged = instruction_decoder::merge_u32s(first, second);

        assert_eq!(number, merged)
    }
}
