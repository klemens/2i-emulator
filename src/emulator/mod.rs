use std::result;

pub mod alu;
pub mod cpu;

#[derive(Debug)]
pub enum Error {
    Cpu(&'static str),
}

pub type Result<T> = result::Result<T, Error>;
