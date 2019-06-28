//! The 2i bus.
//!
//! This module contains a trait for the 8 bit bus used in the 2i and several
//! types implementing it.

use super::{Error, Result};
use std::cell::RefCell;

/// Bus of the 2i.
///
/// Represents an interface of the 2i bus with 8 bit data and addressing.
pub trait Bus {
    fn read(&self, address: u8) -> Result<u8>;
    fn write(&self, address: u8, value: u8) -> Result<()>;
}

/// Ram of the 2i.
///
/// Represents the 8 bit ram of the 2i.
pub struct Ram<'a> {
    memory: RefCell<[u8; 256]>,
    overlays: Vec<(u8, u8, &'a dyn Bus)>,
}

impl<'a> Ram<'a> {
    /// Create a new ram with all addresses initialised to zero.
    pub fn new() -> Ram<'a> {
        Ram::default()
    }

    /// Direct access to the ram wrapped in a RefCell.
    pub fn inspect(&self) -> &RefCell<[u8; 256]> {
        &self.memory
    }

    /// Add a bus as an overlay to the ram.
    ///
    /// When a read or write lies in the given (inclusive) range, the request
    /// is forwarded to the given bus. All overlays are checked in the order
    /// they were added.
    pub fn add_overlay(&mut self, first_address: u8, last_address: u8,
        overlay_bus: &'a dyn Bus) {
        self.overlays.push((first_address, last_address, overlay_bus));
    }
}

impl<'a> Default for Ram<'a> {
    fn default() -> Ram<'a> {
        Ram {
            memory: RefCell::new([0; 256]),
            overlays: Default::default(),
        }
    }
}

impl<'a> Bus for Ram<'a> {
    fn read(&self, address: u8) -> Result<u8> {
        for &(first_address, last_address, bus) in self.overlays.iter() {
            if address >= first_address && address <= last_address {
                return bus.read(address);
            }
        }

        Ok(self.memory.borrow()[address as usize])
    }
    fn write(&self, address: u8, value: u8) -> Result<()> {
        for &(first_address, last_address, bus) in self.overlays.iter() {
            if address >= first_address && address <= last_address {
                return bus.write(address, value);
            }
        }

        self.memory.borrow_mut()[address as usize] = value;
        Ok(())
    }
}

/// IoRegisters of the 2i.
///
/// Represents the input and output registers of the 2i. Reading from an
/// address lower than FC or writing to an address lower than FE will result
/// in an error.
#[derive(Default)]
pub struct IoRegisters {
    input: RefCell<[u8; 4]>,
    output: RefCell<[u8; 2]>,
}

impl IoRegisters {
    /// Create a new IoRegisters with all registers initialised to zero.
    pub fn new() -> IoRegisters {
        IoRegisters::default()
    }

    /// Direct access to the input registers wrapped in a RefCell.
    pub fn inspect_input(&self) -> &RefCell<[u8; 4]> {
        &self.input
    }

    /// Direct access to the output registers wrapped in a RefCell.
    pub fn inspect_output(&self) -> &RefCell<[u8; 2]> {
        &self.output
    }
}

impl Bus for IoRegisters {
    fn read(&self, address: u8) -> Result<u8> {
        if address >= 0xFC {
            Ok(self.input.borrow()[(address - 0xFC) as usize])
        } else {
            Err(Error::Bus("Only supports reading from input registers"))
        }
    }
    fn write(&self, address: u8, value: u8) -> Result<()> {
        if address >= 0xFE {
            self.output.borrow_mut()[(address - 0xFE) as usize] = value;
            Ok(())
        } else if address >= 0xFC {
            Err(Error::Bus("Cannot write to input register"))
        } else {
            Err(Error::Bus("Only supports writing to output registers"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write_memory() {
        let ram = Ram::new();

        ram.write(0, 42).unwrap();
        ram.write(1, 43).unwrap();
        ram.write(2, 44).unwrap();
        ram.write(0xFD, 45).unwrap();
        ram.write(0xFE, 46).unwrap();
        ram.write(0xFF, 47).unwrap();

        assert_eq!(ram.read(0).unwrap(), 42);
        assert_eq!(ram.read(1).unwrap(), 43);
        assert_eq!(ram.read(2).unwrap(), 44);
        assert_eq!(ram.read(3).unwrap(), 0);
        assert_eq!(ram.read(0xFC).unwrap(), 0);
        assert_eq!(ram.read(0xFD).unwrap(), 45);
        assert_eq!(ram.read(0xFE).unwrap(), 46);
        assert_eq!(ram.read(0xFF).unwrap(), 47);
    }

    #[test]
    fn overlay() {
        // We have to declare the overlay first, because otherwise it does not
        // outlive the base and we cannot pass a reference to add_overlay.
        // The order doesn't matter if both are declared inside a struct.
        let overlay = Ram::new();
        let mut base = Ram::new();

        base.add_overlay(2, 3, &overlay);

        base.write(0, 42).unwrap();
        base.write(1, 43).unwrap();
        base.write(2, 44).unwrap();
        base.write(3, 45).unwrap();
        base.write(4, 46).unwrap();

        assert_eq!(base.read(0).unwrap(), 42);
        assert_eq!(base.read(1).unwrap(), 43);
        assert_eq!(base.read(2).unwrap(), 44);
        assert_eq!(base.read(3).unwrap(), 45);
        assert_eq!(base.read(4).unwrap(), 46);

        assert_eq!(overlay.read(0).unwrap(), 0);
        assert_eq!(overlay.read(1).unwrap(), 0);
        assert_eq!(overlay.read(2).unwrap(), 44);
        assert_eq!(overlay.read(3).unwrap(), 45);
        assert_eq!(overlay.read(4).unwrap(), 0);

        assert_eq!(base.inspect().borrow()[0..5], [42, 43, 0, 0, 46]);
    }

    #[test]
    fn io_register() {
        let io = IoRegisters::new();

        io.inspect_input().borrow_mut().clone_from_slice(&[42, 43, 44, 45]);
        assert_eq!(io.read(0xFC).unwrap(), 42);
        assert_eq!(io.read(0xFD).unwrap(), 43);
        assert_eq!(io.read(0xFE).unwrap(), 44);
        assert_eq!(io.read(0xFF).unwrap(), 45);

        io.write(0xFE, 46).unwrap();
        io.write(0xFF, 47).unwrap();
        assert_eq!(io.inspect_output().borrow()[..], [46, 47]);
        assert_eq!(io.read(0xFE).unwrap(), 44);
        assert_eq!(io.read(0xFF).unwrap(), 45);

        assert!(io.read(0).is_err());
        assert!(io.read(0xFB).is_err());
        assert!(io.write(0, 0).is_err());
        assert!(io.write(0xFB, 0).is_err());
        assert!(io.write(0xFC, 0).is_err());
        assert!(io.write(0xFD, 0).is_err());
    }
}
