//! Parse 2i programs.
//!
//! This module contains functions for parsing 2i programs.

use std::io::BufReader;
use std::io::prelude::*;

use regex::Regex;

use super::{Error, Result};
use super::instruction::Instruction;

/// Parse 2i programs in string representation into arrays of `Instruction`s.
///
/// Ignores empty lines and everything after the `#` char. You can use any char
/// other than `0`, `1` and `:` to format your program for improved readability.
///
/// Instructions can optionally be given an explicit address by prefixing them
/// with the binary representation of the address followed by `:`. Instructions
/// without an explicit address are saved at the first unused address.
///
/// # Examples
///
/// ```text
/// # Read value from FC into register 0
///
///        00,00001 00 000|1100 01 01,1100 0
/// 00001: 00,00000 01 000|0000 01 10,0001 0
/// ```
pub fn read_program<R: Read>(reader: R) -> Result<[Instruction; 32]> {
    let instructions = parse_instructions(reader)?;

    // Replace all None values with zero instructions and remove the Option
    let mut final_instructions = [Instruction::default(); 32];
    for (i, instruction) in instructions.iter().enumerate() {
        if let &Some(instruction) = instruction {
            final_instructions[i] = instruction;
        }
    }

    Ok(final_instructions)
}

/// Iterator stored on the stack with variable length and storage size of 2
macro_rules! alternative_2 {
    // TODO: Using a custom iterator instead of once would be more efficient
    ($first:expr) => (
        ::std::iter::once($first).chain(::std::iter::once($first)).take(1)
    );
    ($first:expr, $second:expr) => (
        ::std::iter::once($first).chain(::std::iter::once($second)).take(2)
    );
}

/// Parse 2i programs in string representation and return only the reachable
/// instructions.
///
/// Instructions are considered reachable if there is a chain of instructions
/// starting from the first one at address 0 to it. This also considers
/// conditional jumps.
///
/// For details on the syntax of the string representation see `read_program`.
pub fn read_reachable_program<R: Read>(reader: R) -> Result<Vec<(u8, Instruction)>> {
    #[derive(Clone, Copy)]
    enum S {
        Empty, // Not yet visited
        Visited, // Visited, but instruction is missing (will get default one)
        Instruction(Instruction), // Visited and containing a instruction
    }

    let instructions = parse_instructions(reader)?;
    let mut reachable_instructions = [S::Empty; 32];

    // The instruction at address 0 is reachable by definition if it exists
    reachable_instructions[0] = if let Some(inst) = instructions[0] {
        S::Instruction(inst)
    } else {
        return Err(Error::Parse("No instruction reachable"));
    };

    // Since instructions can jump to earlier addresses, we have to iterate
    // until no new instruction is found.
    let mut finished = false;
    while !finished {
        finished = true;

        for i in 0..reachable_instructions.len() {
            if let S::Instruction(inst) = reachable_instructions[i] {
                let na = inst.get_next_instruction_address();

                // Consider both target addresses for conditional jumps
                let target_addresses = if inst.get_address_control() == 0 {
                    alternative_2!(na)
                } else {
                    alternative_2!(na & !1u8, na | 1u8)
                };

                for addr in target_addresses {
                    let addr = addr as usize;
                    // Only update instruction addresses that were not yet
                    // visited. This ensures that the algorithm terminates
                    if let S::Empty = reachable_instructions[addr] {
                        finished = false;
                        if let Some(inst) = instructions[addr] {
                            reachable_instructions[addr] = S::Instruction(inst);
                        } else {
                            reachable_instructions[addr] = S::Visited;
                        }
                    }
                }
            }
        }
    }

    // Addresses which were visited but did not have a valid instruction get
    // a default one (NOP, JMP 0)
    Ok(reachable_instructions.iter().enumerate().filter_map(|(i,inst)| {
        match *inst {
            S::Empty => None,
            S::Visited => Some((i as u8, Instruction::default())),
            S::Instruction(inst) => Some((i as u8, inst)),
        }
    }).collect())
}

/// Actually parse the instructions from the given reader
///
/// For details on the syntax of the string representation see `read_program`.
fn parse_instructions<R: Read>(reader: R) -> Result<[Option<Instruction>; 32]> {
    let mut instructions = [None; 32];
    let explicit_address = Regex::new(r"^(?P<addr>[01]{5})\s*:\s*(?P<inst>.*)$").unwrap();

    let reader = BufReader::new(reader);
    for line in reader.lines() {
        let line = line?;

        // Remove whitespace and comments that start with #
        let line = match line.find('#') {
            Some(start) => line[..start].trim(),
            None => line.trim(),
        };

        // Ignore empty lines
        if line.is_empty() {
            continue;
        }

        // Check if an explicit address is given
        let (instruction, address) = if line.contains(':') {
            match explicit_address.captures(line) {
                Some(matches) => {
                    let inst = matches.name("inst").unwrap().as_str();
                    let addr = matches.name("addr").unwrap().as_str();
                    (inst, Some(addr))
                }
                None => return Err(Error::Parse("Invalid instruction address")),
            }
        } else {
            (line, None)
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

    Ok(instructions)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parser() {
        let program = parse_instructions(Cursor::new("\
            # Simple program\n\
            \n\
            00000: 00 00001 000000000000000000 # first instruction\n\
          \n       00 00011 000000000000000000# second instruction\n\
            00011: 00 11111 000000000000000000\n\
            # the following instruction ends up in 00010, not 00100!\
          \n       00 00000 000000000000000000\n\
            11111 : 00 00011 | 00 | 000 1111 01 | 01 0100 | 0\n\
        ".to_owned())).unwrap();

        assert_eq!(program.iter().filter_map(|e| *e).collect::<Vec<_>>().as_slice(), &[
            Instruction::new(0b00_00001_000000000000000000).unwrap(),
            Instruction::new(0b00_00011_000000000000000000).unwrap(),
            Instruction::new(0b00_00000_000000000000000000).unwrap(),
            Instruction::new(0b00_11111_000000000000000000).unwrap(),
            Instruction::new(0b00_00011_000001111010101000).unwrap(),
        ]);
    }

    #[test]
    #[should_panic(expected = "Invalid instruction address")]
    fn invalid_address() {
        let _ = parse_instructions(Cursor::new("\
            0 0 0 0 0: 00 00001 000000000000000000\n\
        ".to_owned())).unwrap();
    }

    #[test]
    fn reachable_backjump() {
        let program = Cursor::new("\
            00000: 00 00100 000000000000000000\n\
            00001: 00 11111 000000000000000000\n\
            00010: 00 00001 000000000000000000\n\
            00100: 00 00010 000000000000000000\n\
            11111: 00 00000 000000000000000000\n\
        ".to_owned());
        assert_eq!(read_reachable_program(program).unwrap().as_slice(), &[
            ( 0, Instruction::new(0b00_00100_000000000000000000).unwrap()),
            ( 1, Instruction::new(0b00_11111_000000000000000000).unwrap()),
            ( 2, Instruction::new(0b00_00001_000000000000000000).unwrap()),
            ( 4, Instruction::new(0b00_00010_000000000000000000).unwrap()),
            (31, Instruction::new(0b00_00000_000000000000000000).unwrap()),
        ]);
    }

    #[test]
    fn reachable_address_control() {
        let program = Cursor::new("\
            00000: 11 00010 000000000000000000\n\
            00010: 00 00000 000000000000000000\n\
            00011: 00 00000 000000000000000000\n\
        ".to_owned());
        assert_eq!(read_reachable_program(program).unwrap().as_slice(), &[
            (0, Instruction::new(0b11_00010_000000000000000000).unwrap()),
            (2, Instruction::new(0b00_00000_000000000000000000).unwrap()),
            (3, Instruction::new(0b00_00000_000000000000000000).unwrap()),
        ]);
    }

    #[test]
    #[should_panic(expected = "No instruction reachable")]
    fn reachable_empty() {
        let program = Cursor::new("".to_owned());
        read_reachable_program(program).unwrap();
    }
}
