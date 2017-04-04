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
///
/// # Examples:
///
/// ```
/// use emulator::{Cpu, Instruction, Ram};
///
/// let mut cpu = Cpu::new();
/// let mut ram = Ram::new();
///
/// // R0 = 6
/// let inst = Instruction::new(0b00_00000_00_000_0110_01_01_1100_0).unwrap();
///
/// let _ = cpu.execute_instruction(inst, &mut ram);
/// assert_eq!(6, cpu.inspect_registers()[0]);
/// ```
#[derive(Default)]
pub struct Cpu {
    registers: [u8; 8],
    flag_register: Flags,
    stored_interrupt: bool,
    volatile_interrupt: bool,
}

impl Cpu {
    /// Create a new cpu with all registers and flags set to zero.
    pub fn new() -> Cpu {
        Cpu::default()
    }

    /// Execute the given instruction on the cpu using the given, bus, input
    /// and output. Returns the address of the next instruction and the alu flags.
    pub fn execute_instruction<B: Bus>(&mut self, inst: Instruction, bus: &mut B) -> Result<(usize, Flags)> {
        // Determine alu input a (bus or register)
        let a = if inst.is_alu_input_a_bus() {
            if ! inst.is_bus_enabled() {
                return Err(Error::Cpu("Cannot read from disabled bus"));
            } else if inst.is_bus_writable() {
                return Err(Error::Cpu("Cannot read from bus while it is in write mode"));
            }

            try!(bus.read(self.registers[inst.get_register_address_a()]))
        } else {
            self.registers[inst.get_register_address_a()]
        };

        // Determine alu input b (constant or register)
        let b = if inst.is_alu_input_b_const() {
            inst.get_constant_input()
        } else {
            self.registers[inst.get_register_address_b()]
        };

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
        let next_address = self.calculate_next_instruction_address(inst, flags);

        // Reset interrupts (stored only if MAC = 111)
        self.volatile_interrupt = false;
        if inst.get_address_control() == 0b11 &&
           inst.get_next_instruction_address() & 0b00001 == 0b1 {
            self.stored_interrupt = false;
        }

        Ok((next_address as usize, flags))
    }

    /// Enable the volatile interrupt (MAC 010) for the next instruction executed
    pub fn trigger_volatile_interrupt(&mut self) {
        self.volatile_interrupt = true;
    }

    /// Enable the stored interrupt (MAC 111) until used by any instruction
    pub fn trigger_stored_interrupt(&mut self){
        self.stored_interrupt = true;
    }

    /// Direct access to the registers.
    pub fn inspect_registers(&mut self) -> &mut [u8; 8] {
        &mut self.registers
    }

    /// Direct access to the flag register.
    pub fn inspect_flags(&mut self) -> &mut Flags {
        &mut self.flag_register
    }

    /// Check if the volatile interrupt is active for the next instruction
    pub fn check_volatile_interrupt(&self) -> bool {
        self.volatile_interrupt
    }

    /// Check if the stored interrupt is active
    pub fn check_stored_interrupt(&self) -> bool {
        self.stored_interrupt
    }

    /// Calculate the next instruction address based on the current instruction
    /// and the flags.
    fn calculate_next_instruction_address(&self, inst: Instruction, flags: Flags) -> u8 {
        let next_address = inst.get_next_instruction_address();
        let next_address_base = next_address & 0b11110; // Mask off last bit

        match inst.get_address_control() << 1 | (next_address & 0b00001) {
            0b000 | 0b001 => {
                next_address
            }
            0b010 => {
                next_address_base | self.volatile_interrupt as u8
            }
            0b011 => {
                next_address_base | self.flag_register.carry() as u8
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
                next_address_base | self.stored_interrupt as u8
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
        let na = |inst: u32, flags, carry, volatile_int, stored_int| {
            let inst = Instruction::new(inst << 18).unwrap();
            let mut cpu = Cpu::default();
            cpu.volatile_interrupt = volatile_int;
            cpu.stored_interrupt = stored_int;
            cpu.flag_register = Flags::new(carry, false, false);
            cpu.calculate_next_instruction_address(inst, flags)
        };

        // No modification
        assert_eq!(na(0b00_00000, Flags::new(false, false, false), false, false, false), 0b00000);
        assert_eq!(na(0b00_11100, Flags::new(false, false, false), false, false, false), 0b11100);
        assert_eq!(na(0b00_11111, Flags::new(false, false, false), false, false, false), 0b11111);
        assert_eq!(na(0b00_00000, Flags::new( true,  true,  true),  true, false, false), 0b00000);
        assert_eq!(na(0b00_11100, Flags::new( true,  true,  true),  true, false, false), 0b11100);
        assert_eq!(na(0b00_11111, Flags::new( true,  true,  true),  true, false, false), 0b11111);

        // Set last bit to the volatile interrupt
        assert_eq!(na(0b01_11110, Flags::new(false, false, false), false, false, false), 0b11110);
        assert_eq!(na(0b01_11110, Flags::new(false, false, false), false, true, false), 0b11111);

        // Set last bit to stored carry
        assert_eq!(na(0b01_11111, Flags::new(false, false, false), false, false, false), 0b11110);
        assert_eq!(na(0b01_11111, Flags::new(false, false, false),  true, false, false), 0b11111);

        // Set last bit to carry out
        assert_eq!(na(0b10_11110, Flags::new(false, false, false), false, false, false), 0b11110);
        assert_eq!(na(0b10_11110, Flags::new( true, false, false), false, false, false), 0b11111);

        // Set last bit to zero out
        assert_eq!(na(0b10_11111, Flags::new(false, false, false), false, false, false), 0b11110);
        assert_eq!(na(0b10_11111, Flags::new(false, false,  true), false, false, false), 0b11111);

        // Set last bit to negative out
        assert_eq!(na(0b11_11110, Flags::new(false, false, false), false, false, false), 0b11110);
        assert_eq!(na(0b11_11110, Flags::new(false,  true, false), false, false, false), 0b11111);

        // Set last bit to the stored interrupt
        assert_eq!(na(0b11_11111, Flags::new(false, false, false), false, false, false), 0b11110);
        assert_eq!(na(0b11_11111, Flags::new(false, false, false), false, false,  true), 0b11111);
    }

    #[test]
    fn multiplication() {
        let program: Vec<_> = [
            0b00_00001_00_000_1100_01_01_1100_0, // in:  R0 = FC
            0b00_00010_01_000_0000_01_10_0001_0, //      R0 = (R0)
            0b00_00011_00_001_1101_01_01_1100_0, //      R1 = FD
            0b00_00100_01_001_0000_01_10_0001_0, //      R1 = (R1)
            0b00_00101_00_010_0000_01_00_0011_0, //      R2 = 0
            0b10_00111_00_000_0000_00_00_0001_0, // tst: TEST R0, ZO
            0b00_01000_00_000_1111_01_01_0100_0, //        R0 = R0 + FF, JP add
            0b00_01001_00_001_1110_01_01_1100_0, //        R1 = FF, JP out
            0b00_00101_00_010_0001_01_00_0100_0, // add: R2 = R2 + R1, JP tst
            0b00_00000_11_001_0010_00_00_1100_0, // out: (R1) = R2, JP in
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
