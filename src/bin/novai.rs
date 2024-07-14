use std::{
    env, fs,
    io::{self, Write},
    process::exit,
};

use nova::{compiler, instruction::Instruction, machine::VirtualMachine, natives};

const PROMPT: &str = ">>";

fn main() {
    let args: Vec<String> = env::args().collect::<Vec<String>>();
    if args.len() > 1 {
        run_file(&args[1])
    } else {
        repl()
    }
}

fn repl() {
    let native_functions = natives::common_native_functions();
    let mut interpreter = VirtualMachine::new();
    interpreter.load_natives(native_functions);
    let mut offset = 0 as Instruction;

    loop {
        let mut input = String::new();
        print!("{} ", PROMPT);
        io::stdout().flush().expect("Error writing to output");
        let input_result = io::stdin().read_line(&mut input);
        if input_result.is_err() {
            eprintln!("Error getting input");
            exit(1)
        }

        if input == "quit\r\n" || input == "Quit\r\n" {
            println!("exiting");
            break;
        }

        let program = compiler::compile(&input).unwrap();
        let new_offset = program.instructions.len() as Instruction;

        interpreter.load_program(program);
        interpreter.start_vm(offset);

        offset += new_offset;
    }
}

fn run_file(path: &str) {
    let result = fs::read_to_string(path);
    if let Err(err) = result {
        println!("{}", err);
        return;
    }

    let code = result.unwrap();

    let natives = natives::common_native_functions();
    let mut interpreter = VirtualMachine::new();
    interpreter.load_natives(natives);
    let offset = 0 as Instruction;

    let program = compiler::compile(&code).unwrap();

    interpreter.load_program(program);
    interpreter.start_vm(offset);
}
