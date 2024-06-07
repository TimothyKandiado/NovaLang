use std::{env, fs};

use nova::{compiler, debug::debug_instruction, instruction::Instruction, program::Program};

fn main() {
    let args: Vec<String> = env::args().collect::<Vec<String>>();
    if args.len() > 1 {
        run_file(&args[1], &args);
    } else {
        println!("Error: an argument is required");
        std::process::exit(1);
    }
}

fn run_file(path: &str, _arguments: &Vec<String>) {
    let result = fs::read_to_string(path);
    if let Err(err) = result {
        println!("{}", err);
        return;
    }

    let code = result.unwrap();

    let program = compiler::compile(&code).unwrap();

    debug_code(program);
}

fn debug_code(program: Program) {
    println!("Instructions");
    for (index, _) in program.instructions.iter().enumerate() {
        let instruction_dbg = debug_instruction(&program.instructions, index as Instruction);
        println!("[{}]: {}", index, instruction_dbg);
    }
}