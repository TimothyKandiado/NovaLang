
mod generator;
use nova_tw::language::{Scanner, errors, AstParser};

use crate::program::Program;

#[allow(dead_code)]
pub fn compile(source: &str) -> Result<Program, errors::Error> {
    let tokens = Scanner::new().scan_tokens(source)?;
    let ast = AstParser::new(tokens).parse_ast()?;

    let generator = generator::BytecodeGenerator::new();
    let program = generator.generate_bytecode(&ast);

    if let Err(error) = program {
        return Err(errors::Error::Interpret(error))
    }
    
    Ok(program.unwrap())
}