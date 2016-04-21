use std::result;

pub mod alu;
pub mod bus;
pub mod cpu;

#[derive(Debug)]
pub enum Error {
    Bus(&'static str),
    Cpu(&'static str),
}

pub type Result<T> = result::Result<T, Error>;
