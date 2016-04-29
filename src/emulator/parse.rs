//! Parse 2i programs.
//!
//! This module contains functions for parsing 2i programs.

use std::io::BufReader;
use std::io::prelude::*;

use super::{Error, Result};
use super::instruction::Instruction;

/// Parse 2i programs in string representation into arrays of `Instruction`s.
///
/// Ignores empty lines and lines that start with `#`. You can use any char
/// other than `0` and `1` to format your program for improved readability.
///
/// Instructions can optionally be given an explicit address by prefixing it
/// with the binary representation of the address. Instructions without an
/// address are saved at the first unused address.
///
/// # Example program
///
/// ```text
/// # Read value from FC into register 0
///
///        00,00001 00 000|1100 01 01,0001 0
/// 00001: 00,00000 01 000|0000 01 10,0000 0
/// ```
pub fn read_program<R: Read>(reader: R) -> Result<[Instruction; 32]> {
    let mut instructions = [None; 32];

    let reader = BufReader::new(reader);
    for line in reader.lines() {
        let line = line.expect("Error while reading program.");

        // Ignore comments that start with #
        if line.starts_with("#") || line.is_empty() {
            continue;
        }

        // Check if an explicit address is given
        let parts: Vec<_> = line.splitn(2, ": ").collect();
        let (instruction, address) = if parts.len() == 2 {
            (parts[1], Some(parts[0]))
        } else {
            (parts[0], None)
        };

        // Parse Instruction
        let raw_inst = convert_binary_string_to_int(&instruction);
        let instruction = try!(Instruction::new(raw_inst));

        if let Some(address) = address {
            // Parse specified address
            let address = convert_binary_string_to_int(&address) as usize;
            if address >= 32 {
                return Err(Error::Parse("Specified instruction address too big"));
            }

            if instructions[address].is_none() {
                instructions[address] = Some(instruction);
            } else {
                return Err(Error::Parse("Two instructions with the same address"));
            }
        } else {
            // Find the next free address
            if let Some(address) = instructions.iter().position(|i| i.is_none()) {
                instructions[address] = Some(instruction);
            } else {
                return Err(Error::Parse("Too many instructions in this program"));
            }
        }
    }

    // Replace all None values with zero instructions and remove the Option
    let mut final_instructions = [Instruction::new(0).unwrap(); 32];
    for (i, instruction) in instructions.iter().enumerate() {
        if let &Some(instruction) = instruction {
            final_instructions[i] = instruction;
        }
    }

    Ok(final_instructions)
}

/// Convert a binary string to a u32 ignoring any chars other than 0 and 1
///
/// If the string contains more than 32 valid bits, the excess bits at the
/// beginning are ignored.
fn convert_binary_string_to_int(s: &str) -> u32 {
    let mut result = 0u32;

    for bit in s.chars().filter_map(|c| {
        match c {
            '0' => Some(0),
            '1' => Some(1),
            _ => None,
        }
    }) {
        result = result << 1 | bit;
    }

    result
}
