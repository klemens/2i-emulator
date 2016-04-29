use std::fmt;
use std::result;

pub mod alu;
pub mod bus;
pub mod cpu;
pub mod instruction;
pub mod parse;

// Re-exports
pub use cpu::Cpu;
pub use instruction::Instruction;
pub use bus::{IoRegisters, Ram};

#[derive(Debug)]
pub enum Error {
    Bus(&'static str),
    Cpu(&'static str),
    Instruction(&'static str),
    Parse(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::Bus(s) => write!(f, "Bus error: {}", s),
            &Error::Cpu(s) => write!(f, "Cpu error: {}", s),
            &Error::Instruction(s) => write!(f, "Instruction error: {}", s),
            &Error::Parse(s) => write!(f, "Parse error: {}", s),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
