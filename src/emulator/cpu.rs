//! The 2i cpu.
//!
//! This module contains the cpu used in the 2i.

use super::{Result, Error};
use super::bus::Bus;
use super::instruction::Instruction;

/// Cpu of the 2i.
///
/// Represents the 8 bit cpu of the 2i with 8 registers that are 8 bit wide and
/// the three status registers (carry, negative, zero).
pub struct Cpu {
    registers: [u8; 8],
    flag_register: (bool, bool, bool),
}

impl Cpu {
    /// Create a new cpu with all registers and flags set to zero.
    pub fn new() -> Cpu {
        Cpu {
            registers: [0; 8],
            flag_register: (false, false, false),
        }
    }

    /// Execute the given instruction on the cpu using the given alu, bus,
    /// input and output. Returns the address of the next instruction.
    pub fn execute_instruction<A, B>(&mut self, inst: Instruction, alu: A,
        bus: &mut B) -> Result<u8>
        where A: Fn(u8, u8, u8, bool) -> (u8, (bool, bool, bool)),
              B: Bus {
        let a;
        let b;

        // Determine alu input a (bus or register)
        if inst.is_alu_input_a_bus() {
            if ! inst.is_bus_enabled() {
                return Err(Error::Cpu("Cannot read from disabled bus"));
            } else if inst.is_bus_writable() {
                return Err(Error::Cpu("Cannot read from bus while it is in write mode"));
            }

            a = try!(bus.read(self.registers[inst.get_register_address_a()]));
        } else {
            a = self.registers[inst.get_register_address_a()];
        }

        // Determine alu input b (constant or register)
        if inst.is_alu_input_b_const() {
            let mut constant = inst.get_constant_input();
            if constant & 0b1000 != 0 {
                // Set bits 4-7 to one if bit 3 is set
                constant |= 0b11110000;
            }
            b = constant;
        } else {
            b = self.registers[inst.get_register_address_b()];
        }

        // Calculate result using alu
        let (result, flags) = alu(inst.get_alu_instruction(), a, b, self.flag_register.0);

        // Write result to registers
        if inst.should_write_register() {
            if inst.should_write_register_b() {
                self.registers[inst.get_register_address_b()] = result;
            } else {
                self.registers[inst.get_register_address_a()] = result;
            }
        }

        // Write results to the bus
        if inst.is_bus_enabled() && inst.is_bus_writable() {
            try!(bus.write(self.registers[inst.get_register_address_a()], result));
        }

        // Store flags in the flag register
        if inst.should_store_flags() {
            self.flag_register = flags;
        }

        // Calculate and return the next instruction address
        Ok(Cpu::calculate_next_instruction_address(inst, flags, self.flag_register.0))
    }

    /// Calculate the next instruction address based on the current instruction
    /// and the flags.
    fn calculate_next_instruction_address(inst: Instruction,
        flags: (bool, bool, bool), stored_carry: bool) -> u8 {
        let next_address = inst.get_next_instruction_address();
        let next_address_base = next_address & 0b11110; // Mask off last bit

        match inst.get_address_control() << 1 | (next_address & 0b00001) {
            0b000 | 0b001 => {
                next_address
            }
            0b010 => {
                next_address_base | 0b00001
            }
            0b011 => {
                next_address_base | stored_carry as u8
            }
            0b100 => {
                next_address_base | flags.0 as u8
            }
            0b101 => {
                next_address_base | flags.2 as u8
            }
            0b110 => {
                next_address_base | flags.1 as u8
            }
            0b111 => {
                next_address_base
            }
            _ => {
                panic!("Invlid address control")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::emulator::{Error, Result};
    use ::emulator::alu::calculate;
    use ::emulator::bus::Bus;
    use ::emulator::instruction::Instruction;

    #[test]
    fn address_calculation() {
        // Helper function to keep asserts short
        let na = |inst: u32, flag_register, carry| {
            let inst = Instruction::new(inst << 18).unwrap();
            let result = Cpu::calculate_next_instruction_address(inst, flag_register, carry);
            result
        };

        // No modification
        assert_eq!(na(0b00_00000, (false, false, false), false), 0b00000);
        assert_eq!(na(0b00_11100, (false, false, false), false), 0b11100);
        assert_eq!(na(0b00_11111, (false, false, false), false), 0b11111);
        assert_eq!(na(0b00_00000, ( true,  true,  true),  true), 0b00000);
        assert_eq!(na(0b00_11100, ( true,  true,  true),  true), 0b11100);
        assert_eq!(na(0b00_11111, ( true,  true,  true),  true), 0b11111);

        // Set last bit to 1
        assert_eq!(na(0b01_11110, (false, false, false), false), 0b11111);
        assert_eq!(na(0b01_11110, ( true,  true,  true),  true), 0b11111);

        // Set last bit to stored carry
        assert_eq!(na(0b01_11111, (false, false, false), false), 0b11110);
        assert_eq!(na(0b01_11111, (false, false, false),  true), 0b11111);

        // Set last bit to carry out
        assert_eq!(na(0b10_11110, (false, false, false), false), 0b11110);
        assert_eq!(na(0b10_11110, ( true, false, false), false), 0b11111);

        // Set last bit to zero out
        assert_eq!(na(0b10_11111, (false, false, false), false), 0b11110);
        assert_eq!(na(0b10_11111, (false, false,  true), false), 0b11111);

        // Set last bit to negative out
        assert_eq!(na(0b11_11110, (false, false, false), false), 0b11110);
        assert_eq!(na(0b11_11110, (false,  true, false), false), 0b11111);

        // Set last bit to 0
        assert_eq!(na(0b11_11111, (false, false, false), false), 0b11110);
        assert_eq!(na(0b11_11111, (false,  true, false), false), 0b11110);
    }

    #[test]
    fn multiplication() {
        let program: Vec<_> = [
            0b00_00001_00_000_1100_01_01_0001_0, // in:  R0 = FC
            0b00_00010_01_000_0000_01_10_0000_0, //      R0 = (R0)
            0b00_00011_00_001_1101_01_01_0001_0, //      R1 = FD
            0b00_00100_01_001_0000_01_10_0000_0, //      R1 = (R1)
            0b00_00101_00_010_0000_01_00_0011_0, //      R2 = 0
            0b10_00111_00_000_0000_00_00_0000_0, // tst: TEST R0, ZO
            0b00_01000_00_000_1111_01_01_0100_0, //        R0 = R0 + FF, JP add
            0b00_01001_00_001_1110_01_01_0001_0, //        R1 = FF, JP out
            0b00_00101_00_010_0001_01_00_0100_0, // add: R2 = R2 + R1, JP tst
            0b00_00000_11_001_0010_00_00_0001_0, // out: (R1) = R2, JP in
        ].iter().map(|&i| Instruction::new(i)).collect();

        let mult = |a, b, steps| -> u8 {
            let mut next_instruction_address = 0;
            let mut bus = IoBus {
                input: [a, b, 0, 0],
                output: [0, 0],
            };
            let mut cpu = Cpu::new();

            for _ in 0..steps {
                let inst = program[next_instruction_address].unwrap();
                next_instruction_address = cpu.execute_instruction(inst,
                    calculate, &mut bus).unwrap() as usize;
            }

            bus.output[0]
        };

        // Special cases
        assert_eq!(mult(0, 0, 8), 0);
        assert_eq!(mult(1, 0, 11), 0);
        assert_eq!(mult(0, 1, 8), 0);
        assert_eq!(mult(1, 1, 11), 1);

        // Non-overflowing calculations
        assert_eq!(mult(3, 7, 17), 21);
        assert_eq!(mult(7, 3, 29), 21);
        assert_eq!(mult(22, 11, 74), 242);

        // Overflowing calculations
        assert_eq!(mult(22, 12, 74), 8);
        assert_eq!(mult(128, 64, 392), 0);
        assert_eq!(mult(142, 142, 434), 196);
    }

    /// Mock bus used for simulating io in tests
    struct IoBus {
        input: [u8; 4],
        output: [u8; 2],
    }
    impl Bus for IoBus {
        fn read(&self, address: u8) -> Result<u8> {
            if address >= 0xFC {
                Ok(self.input[(address - 0xFC) as usize])
            } else {
                Err(Error::Bus("Only supports reading from input register"))
            }
        }
        fn write(&mut self, address: u8, value: u8) -> Result<()> {
            if address >= 0xFE {
                self.output[(address - 0xFE) as usize] = value;
                Ok(())
            } else {
                Err(Error::Bus("Only supports writing to output register"))
            }
        }
    }
}
