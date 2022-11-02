use crate::interpreter::{Interpreter, InterpreterResult, InterpreterState};

#[derive(Debug)]
pub struct InterpreterMismatchedBracketsError {
	instruction_ptr: usize,
	missing_brackets: usize,
}

#[derive(Debug)]
pub enum InterpreterErrorReason {
	PtrOutOfBounds(usize),
	ValOutOfBounds { data_ptr: usize, delta: i8 },
	InvalidChar,
	StackUnderflow,
	HaltedMachine,
	MismatchedBrackets(InterpreterMismatchedBracketsError),
	UnprintableByte(u8),
}

pub struct InterpreterError {
	pub reason: InterpreterErrorReason,
}

impl InterpreterError {
	pub fn to_result(self) -> InterpreterResult {
		Err(self)
	}

	pub fn ptr_out_of_bounds(interpreter: &Interpreter) -> Self {
		let ptr: usize = interpreter.data_ptr;

		Self {
			reason: InterpreterErrorReason::PtrOutOfBounds(ptr),
		}
	}

	pub fn val_out_of_bounds(interpreter: &Interpreter, delta: i8) -> Self {
		Self {
			reason: InterpreterErrorReason::ValOutOfBounds {
				data_ptr: interpreter.data_ptr,
				delta,
			},
		}
	}

	pub fn invalid_char() -> Self {
		InterpreterError {
			reason: InterpreterErrorReason::InvalidChar,
		}
	}

	pub fn stack_underflow() -> Self {
		InterpreterError {
			reason: InterpreterErrorReason::StackUnderflow,
		}
	}

	pub fn halted_machine() -> Self {
		InterpreterError {
			reason: InterpreterErrorReason::HaltedMachine,
		}
	}

	pub fn mismatched_brackets(interpreter: &Interpreter) -> Self {
		let instruction_ptr = interpreter.instruction_ptr;
		if let InterpreterState::Skipping(missing_brackets) = interpreter.state {
			InterpreterError {
				reason: InterpreterErrorReason::MismatchedBrackets(InterpreterMismatchedBracketsError {
					instruction_ptr,
					missing_brackets,
				}),
			}
		} else {
			panic!("Not in a skipping state");
		}
	}

	pub fn unprintable_byte(byte: u8) -> Self {
		InterpreterError {
			reason: InterpreterErrorReason::UnprintableByte(byte),
		}
	}
}


pub fn read_byte() -> Option<u8> {
	let mut s = String::new();
	std::io::stdin().read_line(&mut s).ok()?;
	let first_char = s.chars().next()?;

	if !first_char.len_utf8() == 1 {
		return None;
	}

	let mut array = [0u8; 1];
	first_char.encode_utf8(&mut array).bytes().next()
}

pub fn print_char(byte: u8) -> Option<String> {
	let byte_vec: Vec<u8> = vec![byte];
	let string = String::from_utf8(byte_vec).ok()?;

	print!("{string}");
	Some(string)
}
