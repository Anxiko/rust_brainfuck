pub mod error;
mod io;
mod math_utils;
mod memory;

use error::InterpreterError;
use crate::InterpreterSymbol;
use crate::symbol::InterpreterInstruction;
use memory::InterpreterMemory;

const MEM_SIZE: usize = 30_000usize;

#[derive(Debug)]
pub struct Interpreter {
	memory: InterpreterMemory,
	data_ptr: usize,
	instruction_ptr: usize,
	stack: Vec<usize>,
	state: InterpreterState,
}

pub type InterpreterResult = Result<(), InterpreterError>;

#[derive(Debug)]
pub enum InterpreterState {
	Running,
	Skipping(usize),
	Halted,
}

impl Interpreter {
	pub fn new() -> Self {
		Interpreter {
			memory: InterpreterMemory::new(),
			data_ptr: 0usize,
			instruction_ptr: 0usize,
			stack: Vec::new(),
			state: InterpreterState::Running,
		}
	}

	pub fn get_instruction_ptr(&self) -> usize {
		self.instruction_ptr
	}

	pub fn is_halted(&self) -> bool {
		matches!(self.state, InterpreterState::Halted)
	}

	fn read_memory(&self) -> Result<u8, InterpreterError> {
		if let Ok(value) = self.memory.read(self.data_ptr) {
			Ok(value)
		} else {
			Err(InterpreterError::ptr_out_of_bounds_from_interpreter(self))
		}
	}

	fn write_memory(&mut self, value: u8) -> Result<(), InterpreterError> {
		self.memory.write(self.data_ptr, value).map_err(
			|()| InterpreterError::ptr_out_of_bounds_from_interpreter(self)
		)
	}

	fn move_right(&mut self) -> InterpreterResult {
		if self.data_ptr + 1 < MEM_SIZE {
			self.data_ptr += 1;
			Ok(())
		} else {
			InterpreterError::ptr_out_of_bounds_from_interpreter(self).to_result()
		}
	}

	pub fn interpret_symbol(&mut self, symbol: &InterpreterSymbol) -> InterpreterResult {
		let state = &self.state;

		match (state, symbol) {
			(InterpreterState::Halted, _) => InterpreterError::halted_machine().to_result(),
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
				InterpreterError::mismatched_brackets(self).to_result()
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

	fn move_left(&mut self) -> InterpreterResult {
		if self.data_ptr > 0 {
			self.data_ptr -= 1;
			Ok(())
		} else {
			InterpreterError::ptr_out_of_bounds_from_interpreter(self).to_result()
		}
	}

	fn delta_data_cell(&mut self, delta: i8) -> InterpreterResult {
		let val = self.read_memory()?;
		let new_val = math_utils::safe_delta_u8(val, delta).map_err(
			|delta_error|
				InterpreterError::val_out_of_bounds(self.data_ptr, delta_error.right)
		)?;
		self.write_memory(new_val)
	}


	fn increment_cell(&mut self) -> InterpreterResult {
		self.delta_data_cell(1)
	}

	fn decrement_cell(&mut self) -> InterpreterResult {
		self.delta_data_cell(-1)
	}


	fn print_ptr(&mut self) -> InterpreterResult {
		if let Ok(val) = self.read_memory() {
			if let Some(_printed_string) = error::print_char(val) {
				Ok(())
			} else {
				InterpreterError::unprintable_byte(val).to_result()
			}
		} else {
			InterpreterError::ptr_out_of_bounds_from_interpreter(self).to_result()
		}
	}

	fn read_ptr(&mut self) -> InterpreterResult {
		if let Some(byte) = error::read_byte() {
			self.write_memory(byte)
		} else {
			InterpreterError::invalid_char().to_result()
		}
	}

	fn enter_loop(&mut self) -> InterpreterResult {
		if let Ok(val) = self.read_memory() {
			let next_state = if val != 0 {
				self.stack.push(self.instruction_ptr);
				InterpreterState::Running
			} else {
				InterpreterState::Skipping(1)
			};
			self.state = next_state;
			Ok(())
		} else {
			InterpreterError::ptr_out_of_bounds_from_interpreter(self).to_result()
		}
	}

	fn exit_loop(&mut self) -> InterpreterResult {
		if let Some(loop_ptr) = self.stack.pop() {
			self.instruction_ptr = loop_ptr;
			Ok(())
		} else {
			InterpreterError::stack_underflow().to_result()
		}
	}

	fn next_instruction(&mut self) {
		self.instruction_ptr += 1;
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
}