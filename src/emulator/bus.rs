//! The 2i bus.
//!
//! This module contains a trait for the 8 bit bus used in the 2i and several
//! types implementing it.

use super::Result;
use std::cell::RefCell;

/// Bus of the 2i.
///
/// Represents an interface of the 2i bus with 8 bit data and addressing.
pub trait Bus {
    fn read(&self, address: u8) -> Result<u8>;
    fn write(&mut self, address: u8, value: u8) -> Result<()>;
}

/// Ram of the 2i.
///
/// Represents the 8 bit ram of the 2i.
pub struct Ram<'a> {
    memory: [u8; 256],
    overlays: Vec<(u8, u8, &'a RefCell<Box<Bus>>)>,
}

impl<'a> Ram<'a> {
    /// Create a new ram with all addresses initialised to zero.
    pub fn new() -> Ram<'a> {
        Ram {
            memory: [0; 256],
            overlays: Vec::new(),
        }
    }

    /// Access the underlying store as a slice.
    pub fn inspect(&mut self) -> &mut [u8; 256] {
        &mut self.memory
    }

    /// Add a bus as an overlay to the ram.
    ///
    /// When a read or write lies in the given (inclusive) range, the request
    /// is forwarded to the given bus. All overlays are checked in the order
    /// they were added.
    pub fn add_overlay(&mut self, first_address: u8, last_address: u8,
        overlay_bus: &'a RefCell<Box<Bus>>) {
        self.overlays.push((first_address, last_address, overlay_bus));
    }
}

impl<'a> Bus for Ram<'a> {
    fn read(&self, address: u8) -> Result<u8> {
        for &(first_address, last_address, bus) in self.overlays.iter() {
            if address >= first_address && address <= last_address {
                return bus.borrow().read(address);
            }
        }

        Ok(self.memory[address as usize])
    }
    fn write(&mut self, address: u8, value: u8) -> Result<()> {
        for &(first_address, last_address, bus) in self.overlays.iter() {
            if address >= first_address && address <= last_address {
                return bus.borrow_mut().write(address, value);
            }
        }

        self.memory[address as usize] = value;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn read_write_memory() {
        let mut ram = Ram::new();

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
        let overlay: RefCell<Box<Bus>> = RefCell::new(Box::new(Ram::new()));
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

        assert_eq!(overlay.borrow().read(0).unwrap(), 0);
        assert_eq!(overlay.borrow().read(1).unwrap(), 0);
        assert_eq!(overlay.borrow().read(2).unwrap(), 44);
        assert_eq!(overlay.borrow().read(3).unwrap(), 45);
        assert_eq!(overlay.borrow().read(4).unwrap(), 0);

        assert_eq!(base.inspect()[0..5], [42, 43, 0, 0, 46]);
    }
}
