//! The 2i cpu.
//!
//! This module contains the cpu used in the 2i.

use super::{Error, Result};
use super::alu::{Alu, Flags};
use super::bus::Bus;
use super::instruction::Instruction;

/// Cpu of the 2i.
///
/// Represents the 8 bit cpu of the 2i with 8 registers that are 8 bit wide and
/// the three status registers (carry, negative, zero).
pub struct Cpu {
    registers: [u8; 8],
    flag_register: Flags,
}

impl Cpu {
    /// Create a new cpu with all registers and flags set to zero.
    pub fn new() -> Cpu {
        Cpu {
            registers: [0; 8],
            flag_register: Flags::default(),
        }
    }

    /// Execute the given instruction on the cpu using the given, bus,
    /// input and output. Returns the address of the next instruction.
    pub fn execute_instruction<B: Bus>(&mut self, inst: Instruction, bus: &mut B) -> Result<(usize, Flags)> {
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
            b = inst.get_constant_input();
        } else {
            b = self.registers[inst.get_register_address_b()];
        }

        // Calculate result using alu
        let (result, flags) = Alu::calculate(inst.get_alu_instruction(), a, b,
            self.flag_register.carry());

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
        Ok((Cpu::calculate_next_instruction_address(inst, flags, self.flag_register.carry()) as usize, flags))
    }

    /// Direct access to the registers.
    pub fn inspect_registers(&mut self) -> &mut [u8; 8] {
        &mut self.registers
    }

    /// Direct access to the flag register.
    pub fn inspect_flags(&mut self) -> &mut Flags {
        &mut self.flag_register
    }

    /// Calculate the next instruction address based on the current instruction
    /// and the flags.
    fn calculate_next_instruction_address(inst: Instruction, flags: Flags,
        stored_carry: bool) -> u8 {
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
                next_address_base | flags.carry() as u8
            }
            0b101 => {
                next_address_base | flags.zero() as u8
            }
            0b110 => {
                next_address_base | flags.negative() as u8
            }
            0b111 => {
                next_address_base
            }
            _ => {
                panic!("Invalid address control")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alu::Flags;
    use bus::IoRegisters;
    use instruction::Instruction;

    #[test]
    fn address_calculation() {
        // Helper function to keep asserts short
        let na = |inst: u32, flag_register, carry| {
            let inst = Instruction::new(inst << 18).unwrap();
            let result = Cpu::calculate_next_instruction_address(inst, flag_register, carry);
            result
        };

        // No modification
        assert_eq!(na(0b00_00000, Flags::new(false, false, false), false), 0b00000);
        assert_eq!(na(0b00_11100, Flags::new(false, false, false), false), 0b11100);
        assert_eq!(na(0b00_11111, Flags::new(false, false, false), false), 0b11111);
        assert_eq!(na(0b00_00000, Flags::new( true,  true,  true),  true), 0b00000);
        assert_eq!(na(0b00_11100, Flags::new( true,  true,  true),  true), 0b11100);
        assert_eq!(na(0b00_11111, Flags::new( true,  true,  true),  true), 0b11111);

        // Set last bit to 1
        assert_eq!(na(0b01_11110, Flags::new(false, false, false), false), 0b11111);
        assert_eq!(na(0b01_11110, Flags::new( true,  true,  true),  true), 0b11111);

        // Set last bit to stored carry
        assert_eq!(na(0b01_11111, Flags::new(false, false, false), false), 0b11110);
        assert_eq!(na(0b01_11111, Flags::new(false, false, false),  true), 0b11111);

        // Set last bit to carry out
        assert_eq!(na(0b10_11110, Flags::new(false, false, false), false), 0b11110);
        assert_eq!(na(0b10_11110, Flags::new( true, false, false), false), 0b11111);

        // Set last bit to zero out
        assert_eq!(na(0b10_11111, Flags::new(false, false, false), false), 0b11110);
        assert_eq!(na(0b10_11111, Flags::new(false, false,  true), false), 0b11111);

        // Set last bit to negative out
        assert_eq!(na(0b11_11110, Flags::new(false, false, false), false), 0b11110);
        assert_eq!(na(0b11_11110, Flags::new(false,  true, false), false), 0b11111);

        // Set last bit to 0
        assert_eq!(na(0b11_11111, Flags::new(false, false, false), false), 0b11110);
        assert_eq!(na(0b11_11111, Flags::new(false,  true, false), false), 0b11110);
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
        ].iter().map(|&i| Instruction::new(i).unwrap()).collect();

        let mult = |a, b, steps| -> u8 {
            let mut next_instruction_address = 0;
            let mut cpu = Cpu::new();
            let mut bus = IoRegisters::new();

            bus.inspect_input().borrow_mut()[0..2].clone_from_slice(&[a, b]);

            for _ in 0..steps {
                let inst = program[next_instruction_address];
                next_instruction_address = cpu.execute_instruction(inst,
                    &mut bus).unwrap().0;
            }

            // The return is necessary because of the following issue:
            // https://github.com/rust-lang/rust/issues/31439
            return bus.inspect_output().borrow()[0];
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
}
