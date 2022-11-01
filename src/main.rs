mod interpreter;
mod symbol;

use std::env;
use std::fs;

use interpreter::Interpreter;
use interpreter::InterpreterResult;
use symbol::InterpreterSymbol;


fn read_file(filename: &str) -> Vec<char> {
    let file_contents: String = fs::read_to_string(
        filename
    ).expect(&format!("Open file: {filename}"));

    file_contents.chars().collect()
}

fn read_instruction(characters: &Vec<char>, bf_interpreter: &Interpreter) -> InterpreterSymbol {
    let character = characters.get(bf_interpreter.get_instruction_ptr());

    InterpreterSymbol::from_char(character)
}

fn run_interpreter(characters: Vec<char>) -> InterpreterResult {
    let mut bf_interpreter = Interpreter::new();
    while !bf_interpreter.is_halted() {
        let symbol = read_instruction(&characters, &bf_interpreter);
        bf_interpreter.interpret_symbol(&symbol)?;
    }
    Ok(())
}

fn print_usage(program_name: &str) -> () {
    println!("Usage: {program_name} brainfuck.bf");
}

fn extract_filename() -> Option<String> {
    let args: Vec<String> = env::args().collect();
    match &args[..] {
        [_, filename] => Some(filename.clone()),
        [] => {
            print_usage("brainfuck.exe");
            None
        }
        [program_name, ..] => {
            print_usage(&program_name);
            None
        }
    }
}

fn main() {
    if let Some(filename) = extract_filename() {
        let characters = read_file(&filename);
        let result = run_interpreter(characters);
        match result {
            Ok(_final_interpreter) => {
                println!("Finished OK!");
            }
            Err(_err) => {
                println!("Finished with error!");
            }
        }
    }
}
