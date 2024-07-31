use std::{env, fs};

use nova::{
    bytecode::OpCode,
    compiler,
    debug::debug_instruction,
    instruction::instruction_decoder,
    program::Program,
};

fn main() {
    let args: Vec<String> = env::args().collect::<Vec<String>>();
    if args.len() > 1 {
        run_file(&args[1], &args);
    } else {
        println!("Error: an argument is required");
        std::process::exit(1);
    }
}

fn run_file(path: &str, _arguments: &[String]) {
    let result = fs::read_to_string(path);
    if let Err(err) = result {
        println!("{}", err);
        return;
    }

    let code = result.unwrap();

    let program = compiler::compile(&code).unwrap();

    debug_code(&program);
    debug_immutables(&program);
}

fn debug_code(program: &Program) {
    println!("Instructions");

    let mut index = 0;

    while index < program.instructions.len() {
        let instruction_dbg = debug_instruction(&program.instructions, index as u64);
        println!("[{}]: {}", index, instruction_dbg);

        let code = instruction_decoder::decode_opcode(program.instructions[index]);
        if code == OpCode::LoadFloat32 as u32 {
            index += 1;
        }
        else if code == OpCode::LoadFloat64 as u32 {
            index += 2;
        }
        else if code == OpCode::LoadInt32 as u32 {
            index += 1;
        }
        else if code == OpCode::LoadInt64 as u32 {
            index += 2;
        }

        index += 1;
    }
}

fn debug_immutables(program: &Program) {
    println!("Immutables");

    for (index, novaobject) in program.immutables.iter().enumerate() {
        println!("[{}]: {}", index, novaobject);
    }
}
