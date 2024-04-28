use std::{env, io::{self, Write}, process::exit};

use nova::{compiler, instruction::Instruction, machine::VirtualMachine};

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
    let mut interpreter = VirtualMachine::new();
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
        let new_offset =  program.instructions.len() as Instruction;

        interpreter.load_program(program);
        interpreter.start_vm(offset);

        offset += new_offset;
    }
}

fn run_file(_path: &str) {
    todo!()
}
