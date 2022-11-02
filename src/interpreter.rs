pub mod error;
mod io;
mod math_utils;

use error::InterpreterError;
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

pub enum InterpreterState {
	Running,
	Skipping(usize),
	Halted,
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

	pub fn get_instruction_ptr(&self) -> usize {
		self.instruction_ptr
	}

	pub fn is_halted(&self) -> bool {
		matches!(self.state, InterpreterState::Halted)
	}

	fn move_right(&mut self) -> InterpreterResult {
		if self.data_ptr + 1 < MEM_SIZE {
			self.data_ptr += 1;
			Ok(())
		} else {
			InterpreterError::ptr_out_of_bounds(self)
		}
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
			if let Some(new_val) = math_utils::safe_delta_u8(*mem_ref, delta) {
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
			if let Some(_printed_string) = error::print_char(byte) {
				Ok(())
			} else {
				InterpreterError::unprintable_byte(byte)
			}
		} else {
			InterpreterError::ptr_out_of_bounds(self)
		}
	}

	fn read_ptr(&mut self) -> InterpreterResult {
		if let Some(byte) = error::read_byte() {
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