mod generator;
use nova_tw::language::{errors, AstParser, Scanner};

use crate::program::Program;

#[allow(dead_code)]
pub fn compile(source: &str, filename: &str) -> Result<Program, errors::Error> {
    let scanner = Scanner::new();
    let tokens = scanner.scan_tokens_with_filename(source, filename)?;

    let ast = AstParser::new(tokens).parse_ast()?;

    let generator = generator::BytecodeGenerator::new();
    let program = generator.generate_bytecode(&ast);

    if let Err(error) = program {
        return Err(errors::Error::Interpret(error));
    }

    Ok(program.unwrap())
}
