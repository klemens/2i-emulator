//! This crate implements an emulator for the *Minirechner 2i*
//!
//! The *Minirechner 2i* is a simple 8 bit micro computer that can be
//! programmed using microcode and is used in the computer science hardware
//! course at Leipzig University.

use std::fmt;
use std::io;
use std::result;

pub mod alu;
pub mod bus;
pub mod cpu;
pub mod instruction;
pub mod parse;

// Re-exports
pub use crate::alu::Flags;
pub use crate::cpu::Cpu;
pub use crate::instruction::Instruction;
pub use crate::bus::{Bus, IoRegisters, Ram};

#[derive(Debug)]
pub enum Error {
    Bus(&'static str),
    Cpu(&'static str),
    Instruction(&'static str),
    Parse(&'static str),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &Error::Bus(s) => write!(f, "Bus error: {}", s),
            &Error::Cpu(s) => write!(f, "Cpu error: {}", s),
            &Error::Instruction(s) => write!(f, "Instruction error: {}", s),
            &Error::Parse(s) => write!(f, "Parse error: {}", s),
            &Error::Io(ref s) => write!(f, "IO error: {}", s),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

pub type Result<T> = result::Result<T, Error>;
