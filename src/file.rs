use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    error::Error,
    fmt::Display,
    fs::{self, File},
    io::{BufReader, Write},
};

use crate::{instruction::Instruction, object::{NovaFunction, NovaObject}, program::Program, version};

#[derive(Debug)]
struct FileError {
    description: String,
}

impl Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl Error for FileError {}

pub struct Metadata {
    version_major: Instruction,
    version_minor: Instruction,
    instruction_count: Instruction,
    immutables_count: Instruction,
}

#[repr(u8)]
enum ImmutableKind {
    String,
    NovaFunction,
}

pub fn write_program_file(path: &str, program: &Program) -> Result<(), Box<dyn Error>> {
    let mut buffer = Vec::new();
    let version_major = version::major();
    let version_minor = version::minor();
    let instruction_count = program.instructions.len() as Instruction;
    let immutables_count = program.immutables.len() as Instruction;

    let metadata = Metadata {
        version_major,
        version_minor,
        instruction_count,
        immutables_count,
    };

    write_metadata(metadata, &mut buffer)?;
    write_instructions(program, &mut buffer)?;
    write_immutables(program, &mut buffer)?;

    let mut file = fs::File::create(path)?;
    file.write(&buffer)?;

    Ok(())
}

fn write_metadata(metadata: Metadata, buffer: &mut Vec<u8>) -> Result<(), Box<dyn Error>> {
    buffer.write_u32::<LittleEndian>(metadata.version_major)?;
    buffer.write_u32::<LittleEndian>(metadata.version_minor)?;
    buffer.write_u32::<LittleEndian>(metadata.instruction_count)?;
    buffer.write_u32::<LittleEndian>(metadata.immutables_count)?;

    Ok(())
}

fn write_instructions(program: &Program, buffer: &mut Vec<u8>) -> Result<(), Box<dyn Error>> {
    for &instruction in program.instructions.iter() {
        buffer.write_u32::<LittleEndian>(instruction)?;
    }

    Ok(())
}

fn write_immutables(program: &Program, buffer: &mut Vec<u8>) -> Result<(), Box<dyn Error>> {
    for immutable in program.immutables.iter() {
        match immutable {
            NovaObject::String(string) => {
                buffer.write_u8(ImmutableKind::String as u8)?; // write a type
                let length = string.len();
                buffer.write_u64::<LittleEndian>(length as u64)?; // write size
                let bytes = string.as_bytes();
                buffer.write(bytes)?;
            }

            NovaObject::NovaFunction(function) => {
                buffer.write_u8(ImmutableKind::NovaFunction as u8)?; // write a type
                buffer.write_u32::<LittleEndian>(function.address)?;
                buffer.write_u8(function.arity as u8)?;
                buffer.write_u8(function.is_method as u8)?;
                let length = function.name.len();
                buffer.write_u64::<LittleEndian>(length as u64)?; 
                let bytes = function.name.as_bytes();
                buffer.write(bytes)?;
            }

            NovaObject::None => {
                continue;
            }
        }
    }

    Ok(())
}

pub fn read_program_file(path: &str) -> Result<Program, Box<dyn Error>> {
    let file = fs::File::open(path)?;
    //let mut buffer = Vec::new();
    //file.read(&mut buffer);

    let mut reader = BufReader::new(file);
    let metadata = read_metadata(&mut reader)?;
    let version_major = version::major();
    let version_minor = version::minor();

    if metadata.version_major > version_major {
        return Err(Box::new(FileError {
            description: format!(
                "Version of bytecode {}.{} higher than supported {}.{}",
                metadata.version_major, metadata.version_minor, version_major, version_minor
            ),
        }));
    }

    if metadata.version_minor > version_minor {
        return Err(Box::new(FileError {
            description: format!(
                "Version of bytecode {}.{} higher than supported {}.{}",
                metadata.version_major, metadata.version_minor, version_major, version_minor
            ),
        }));
    }

    let instructions = read_instructions(&mut reader, metadata.instruction_count)?;
    let immutables = read_immutables(&mut reader, metadata.immutables_count)?;

    Ok(Program {
        instructions,
        immutables,
    })
}

fn read_metadata(reader: &mut BufReader<File>) -> Result<Metadata, Box<dyn Error>> {
    let version_major = reader.read_u32::<LittleEndian>()?;
    let version_minor = reader.read_u32::<LittleEndian>()?;
    let instruction_count = reader.read_u32::<LittleEndian>()?;
    let immutables_count = reader.read_u32::<LittleEndian>()?;

    Ok(Metadata {
        version_major,
        version_minor,
        instruction_count,
        immutables_count,
    })
}

pub fn read_instructions(
    reader: &mut BufReader<File>,
    instruction_count: u32,
) -> Result<Vec<u32>, Box<dyn Error>> {
    let mut instructions = Vec::new();
    for _ in 0..instruction_count {
        let instruction = reader.read_u32::<LittleEndian>()?;
        instructions.push(instruction);
    }

    Ok(instructions)
}

pub fn read_immutables(
    reader: &mut BufReader<File>,
    immutables_count: u32,
) -> Result<Vec<NovaObject>, Box<dyn Error>> {
    let mut immutables = Vec::new();
    for _ in 0..immutables_count {
        let immutable_kind = reader.read_u8()?;

        match immutable_kind {
            x if x == ImmutableKind::String as u8 => {
                let length = reader.read_u64::<LittleEndian>()?;
                let mut str_buffer = Vec::with_capacity(length as usize);
                for _ in 0..length {
                    let byte = reader.read_u8()?;
                    str_buffer.push(byte);
                }

                let string = String::from_utf8(str_buffer)?;
                immutables.push(NovaObject::String(Box::new(string)))
            }

            x if x == ImmutableKind::NovaFunction as u8 => {
                let address = reader.read_u32::<LittleEndian>()?;
                let arity = reader.read_u8()? as Instruction;
                let is_method = reader.read_u8()? != 0;
                let length = reader.read_u64::<LittleEndian>()?;
                let mut str_buffer = Vec::with_capacity(length as usize);
                for _ in 0..length {
                    let byte = reader.read_u8()?;
                    str_buffer.push(byte);
                }

                let name = Box::new(String::from_utf8(str_buffer)?);

                immutables.push(NovaObject::NovaFunction(NovaFunction {
                    name,
                    address,
                    arity,
                    is_method,
                }))
            }

            _ => {
                return Err(Box::new(FileError {
                    description: format!(
                        "Cannot read immutable_kind {:?} from file",
                        immutable_kind
                    ),
                }))
            }
        }
    }

    Ok(immutables)
}

#[cfg(test)]
mod file_tests {
    use crate::{
        bytecode::OpCode, instruction::InstructionBuilder, object::NovaObject, program::Program,
    };

    use super::{read_program_file, write_program_file};

    #[test]
    fn test_write_and_read() {
        let program = get_program();
        write_program_file("test.nvc", &program).unwrap();
        let r_program = read_program_file("test.nvc").unwrap();
        assert_eq!(program.instructions, r_program.instructions);
        assert_eq!(program.immutables, r_program.immutables);
    }

    fn get_program() -> Program {
        let immutables = vec![NovaObject::String(Box::new("I am Timothy".to_string()))];

        let instructions = vec![
            InstructionBuilder::new_load_float32_instruction(0),
            10.0f32.to_bits(),
            InstructionBuilder::new_load_float32_instruction(1),
            15.0f32.to_bits(),
            InstructionBuilder::new_binary_op_instruction(OpCode::Add, 0, 0, 1),
            InstructionBuilder::new_print_instruction(0, true),
            InstructionBuilder::new_binary_op_instruction(OpCode::Mod, 0, 0, 1),
            InstructionBuilder::new_print_instruction(0, true),
            InstructionBuilder::new_load_constant_instruction(2, 0),
            InstructionBuilder::new_print_instruction(2, true),
            InstructionBuilder::new_binary_op_instruction(OpCode::Add, 0, 0, 2),
            InstructionBuilder::new_print_instruction(0, true),
            InstructionBuilder::new_halt_instruction(),
        ];
        Program {
            instructions,
            immutables,
        }
    }
}
