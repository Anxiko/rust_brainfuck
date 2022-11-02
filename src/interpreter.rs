use crate::InterpreterSymbol;
use crate::symbol::InterpreterInstruction;

const MEM_SIZE: usize = 30_000usize;

pub struct Interpreter {
	memory: [u8; MEM_SIZE],
	data_ptr: usize,
	instruction_ptr: usize,
	stack: Vec<usize>,
	state: InterpreterState,
}

pub type InterpreterResult = Result<(), InterpreterError>;

#[derive(Debug)]
pub struct InterpreterMismatchedBracketsError {
	instruction_ptr: usize,
	missing_brackets: usize,
}

#[derive(Debug)]
pub enum InterpreterErrorReason {
	PtrOutOfBounds(usize),
	ValOutOfBounds(u8),
	InvalidChar,
	StackUnderflow,
	HaltedMachine,
	MismatchedBrackets(InterpreterMismatchedBracketsError),
	UnprintableByte(u8),
}

pub enum InterpreterState {
	Running,
	Skipping(usize),
	Halted,
}

pub struct InterpreterError {
	pub reason: InterpreterErrorReason,
}

impl InterpreterError {
	fn ptr_out_of_bounds(interpreter: &Interpreter) -> InterpreterResult {
		let ptr: usize = interpreter.data_ptr;

		Err(InterpreterError {
			reason: InterpreterErrorReason::PtrOutOfBounds(ptr),
		})
	}

	fn val_out_of_bounds(interpreter: &Interpreter) -> InterpreterResult {
		let val = interpreter.memory[interpreter.data_ptr];
		Err(InterpreterError {
			reason: InterpreterErrorReason::ValOutOfBounds(val),
		})
	}

	fn invalid_char() -> InterpreterResult {
		Err(InterpreterError {
			reason: InterpreterErrorReason::InvalidChar,
		})
	}

	fn stack_underflow() -> InterpreterResult {
		Err(InterpreterError {
			reason: InterpreterErrorReason::StackUnderflow,
		})
	}

	fn halted_machine() -> InterpreterResult {
		Err(InterpreterError {
			reason: InterpreterErrorReason::HaltedMachine,
		})
	}

	fn mismatched_brackets(interpreter: &Interpreter) -> InterpreterResult {
		let instruction_ptr = interpreter.instruction_ptr;
		if let InterpreterState::Skipping(missing_brackets) = interpreter.state {
			Err(InterpreterError {
				reason: InterpreterErrorReason::MismatchedBrackets(InterpreterMismatchedBracketsError {
					instruction_ptr,
					missing_brackets,
				}),
			})
		} else {
			panic!("Not in a skipping state");
		}
	}

	fn unprintable_byte(byte: u8) -> InterpreterResult {
		Err(InterpreterError {
			reason: InterpreterErrorReason::UnprintableByte(byte),
		})
	}
}

fn safe_delta_u8(left: u8, right: i8) -> Option<u8> {
	if right > 0 {
		left.checked_add(right as u8)
	} else {
		let right: u8 = right.unsigned_abs();
		left.checked_sub(right)
	}
}


fn read_byte() -> Option<u8> {
	let mut s = String::new();
	std::io::stdin().read_line(&mut s).ok()?;
	let first_char = s.chars().next()?;

	if !first_char.len_utf8() == 1 {
		return None;
	}

	let mut array = [0u8; 1];
	first_char.encode_utf8(&mut array).bytes().next()
}

fn print_char(byte: u8) -> Option<String> {
	let byte_vec: Vec<u8> = vec![byte];
	let string = String::from_utf8(byte_vec).ok()?;

	print!("{string}");
	Some(string)
}

impl Interpreter {
	pub fn new() -> Self {
		Interpreter {
			memory: [0u8; MEM_SIZE],
			data_ptr: 0usize,
			instruction_ptr: 0usize,
			stack: Vec::new(),
			state: InterpreterState::Running,
		}
	}

	fn move_right(&mut self) -> InterpreterResult {
		if self.data_ptr + 1 < MEM_SIZE {
			self.data_ptr += 1;
			Ok(())
		} else {
			InterpreterError::ptr_out_of_bounds(self)
		}
	}

	fn move_left(&mut self) -> InterpreterResult {
		if self.data_ptr > 0 {
			self.data_ptr -= 1;
			Ok(())
		} else {
			InterpreterError::ptr_out_of_bounds(self)
		}
	}

	fn mem_ref(&mut self) -> Option<&mut u8> {
		if self.data_ptr < MEM_SIZE {
			Some(&mut self.memory[self.data_ptr])
		} else {
			None
		}
	}

	fn delta_data_cell(&mut self, delta: i8) -> InterpreterResult {
		if let Some(mem_ref) = self.mem_ref() {
			if let Some(new_val) = safe_delta_u8(*mem_ref, delta) {
				*mem_ref = new_val;
				Ok(())
			} else {
				InterpreterError::val_out_of_bounds(self)
			}
		} else {
			InterpreterError::ptr_out_of_bounds(self)
		}
	}

	fn increment_cell(&mut self) -> InterpreterResult {
		self.delta_data_cell(1)
	}

	fn decrement_cell(&mut self) -> InterpreterResult {
		self.delta_data_cell(-1)
	}


	fn print_ptr(&mut self) -> InterpreterResult {
		if let Some(mem_ref) = self.mem_ref() {
			let byte = *mem_ref;
			if let Some(_printed_string) = print_char(byte) {
				Ok(())
			} else {
				InterpreterError::unprintable_byte(byte)
			}
		} else {
			InterpreterError::ptr_out_of_bounds(self)
		}
	}

	fn read_ptr(&mut self) -> InterpreterResult {
		if let Some(byte) = read_byte() {
			if let Some(mem_ref) = self.mem_ref() {
				*mem_ref = byte;
				Ok(())
			} else {
				InterpreterError::ptr_out_of_bounds(self)
			}
		} else {
			InterpreterError::invalid_char()
		}
	}

	fn enter_loop(&mut self) -> InterpreterResult {
		if let Some(byte) = self.mem_ref() {
			let next_state = if *byte != 0 {
				self.stack.push(self.instruction_ptr);
				InterpreterState::Running
			} else {
				InterpreterState::Skipping(1)
			};
			self.state = next_state;
			Ok(())
		} else {
			InterpreterError::ptr_out_of_bounds(self)
		}
	}

	fn exit_loop(&mut self) -> InterpreterResult {
		if let Some(loop_ptr) = self.stack.pop() {
			self.instruction_ptr = loop_ptr;
			Ok(())
		} else {
			InterpreterError::stack_underflow()
		}
	}

	fn next_instruction(&mut self) {
		self.instruction_ptr += 1;
	}

	pub fn get_instruction_ptr(&self) -> usize {
		self.instruction_ptr
	}

	pub fn is_halted(&self) -> bool {
		matches!(self.state, InterpreterState::Halted)
	}

	fn halt(&mut self) {
		self.state = InterpreterState::Halted;
	}

	fn run_instruction(&mut self, instruction: &InterpreterInstruction) -> InterpreterResult {
		let (advance, result) = match instruction {
			InterpreterInstruction::MovePtrRight => (true, self.move_right()),
			InterpreterInstruction::MovePtrLeft => (true, self.move_left()),
			InterpreterInstruction::IncrementPtr => (true, self.increment_cell()),
			InterpreterInstruction::DecrementPtr => (true, self.decrement_cell()),
			InterpreterInstruction::PrintPtr => (true, self.print_ptr()),
			InterpreterInstruction::ReadPtr => (true, self.read_ptr()),
			InterpreterInstruction::LoopStart => (true, self.enter_loop()),
			InterpreterInstruction::LoopEnd => (false, self.exit_loop())
		};
		if advance && result.is_ok() {
			self.next_instruction();
		}
		result
	}

	pub fn interpret_symbol(&mut self, symbol: &InterpreterSymbol) -> InterpreterResult {
		let state = &self.state;

		match (state, symbol) {
			(InterpreterState::Halted, _) => InterpreterError::halted_machine(),
			(InterpreterState::Skipping(skip), InterpreterSymbol::Instruction(InterpreterInstruction::LoopEnd)) => {
				let skip = skip - 1;
				if skip > 0 {
					self.state = InterpreterState::Skipping(skip);
				} else {
					self.state = InterpreterState::Running;
				}
				self.next_instruction();
				Ok(())
			}
			(InterpreterState::Skipping(_), InterpreterSymbol::EOF) => {
				InterpreterError::mismatched_brackets(self)
			}
			(InterpreterState::Skipping(skip), InterpreterSymbol::Instruction(InterpreterInstruction::LoopStart)) => {
				self.state = InterpreterState::Skipping(skip + 1);
				self.next_instruction();
				Ok(())
			}
			(InterpreterState::Skipping(_), _) => {
				self.next_instruction();
				Ok(())
			}
			(InterpreterState::Running, InterpreterSymbol::EOF) => {
				self.halt();
				Ok(())
			}
			(InterpreterState::Running, InterpreterSymbol::Instruction(instruction)) => {
				self.run_instruction(instruction)?;
				Ok(())
			}
			(InterpreterState::Running, InterpreterSymbol::Other(_)) => {
				self.next_instruction();
				Ok(())
			}
		}
	}
}