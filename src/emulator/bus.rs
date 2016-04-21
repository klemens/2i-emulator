//! The 2i bus.
//!
//! This module contains a trait for the 8 bit bus used in the 2i.

use super::Result;

/// Bus of the 2i
///
/// Represents an interface of the 2i bus with 8 bit data and addressing.
pub trait Bus {
    fn read(&self, address: u8) -> Result<u8>;
    fn write(&mut self, address: u8, value: u8) -> Result<()>;
}
