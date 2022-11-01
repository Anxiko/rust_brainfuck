pub enum InterpreterInstruction {
	MovePtrRight,
	MovePtrLeft,
	IncrementPtr,
	DecrementPtr,
	PrintPtr,
	ReadPtr,
	LoopStart,
	LoopEnd,
}

pub enum InterpreterSymbol {
	Instruction(InterpreterInstruction),
	EOF,
	Other(char),
}

impl InterpreterSymbol {
	pub fn from_char(c: Option<&char>) -> Self {
		if let Some(c) = c {
			match c {
				'>' => InterpreterSymbol::Instruction(InterpreterInstruction::MovePtrRight),
				'<' => InterpreterSymbol::Instruction(InterpreterInstruction::MovePtrLeft),
				'+' => InterpreterSymbol::Instruction(InterpreterInstruction::IncrementPtr),
				'-' => InterpreterSymbol::Instruction(InterpreterInstruction::DecrementPtr),
				'.' => InterpreterSymbol::Instruction(InterpreterInstruction::PrintPtr),
				',' => InterpreterSymbol::Instruction(InterpreterInstruction::ReadPtr),
				'[' => InterpreterSymbol::Instruction(InterpreterInstruction::LoopStart),
				']' => InterpreterSymbol::Instruction(InterpreterInstruction::LoopEnd),
				any_c => InterpreterSymbol::Other(any_c.clone()),
			}
		} else {
			InterpreterSymbol::EOF
		}
	}
}