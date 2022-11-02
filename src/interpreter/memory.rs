use std::fmt::{Debug, Display, Formatter};

const MEMORY_SIZE: usize = 30_000;

pub(super) struct InterpreterMemory {
	memory: [u8; MEMORY_SIZE],
	highest_written: usize,
}

impl InterpreterMemory {
	pub fn new() -> Self {
		InterpreterMemory {
			memory: [0u8; MEMORY_SIZE],
			highest_written: 0,
		}
	}

	pub fn read(&self, address: usize) -> Result<u8, ()> {
		if address < MEMORY_SIZE {
			Ok(self.memory[address])
		} else {
			Err(())
		}
	}

	pub fn write(&mut self, address: usize, value: u8) -> Result<(), ()> {
		if address < MEMORY_SIZE {
			self.memory[address] = value;
			if address > self.highest_written {
				self.highest_written = address;
			}

			Ok(())
		} else {
			Err(())
		}
	}
}

impl Display for InterpreterMemory {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "[")?;
		for idx in 0..self.highest_written {
			let byte = self.memory[idx];
			write!(f, "{byte:02X}.")?;
		}
		write!(f, "]")
	}
}

impl Debug for InterpreterMemory {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		Display::fmt(self, f)
	}
}