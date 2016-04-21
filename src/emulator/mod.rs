use std::fmt;
use std::result;

pub mod alu;
pub mod bus;
pub mod cpu;
pub mod instruction;

#[derive(Debug)]
pub enum Error {
    Bus(&'static str),
    Cpu(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::Bus(s) => write!(f, "Bus error: {}", s),
            &Error::Cpu(s) => write!(f, "Cpu error: {}", s),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
