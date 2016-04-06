//! The 2i cpu.
//!
//! This module contains the cpu used in the 2i.

/// Instruction of the 2i.
///
/// Represents a 25 bit wide instruction that the 2i uses. Provides some
/// conveniance methods to extract all different parts.
pub struct Instruction {
    instruction: u32,
}

/// Cpu of the 2i.
///
/// Represents the 8 bit cpu of the 2i with 8 registers that are 8 bit wide and
/// the three status registers (carry, negative, zero).
pub struct Cpu {
    registers: [u8; 8],
    flags: (bool, bool, bool),
}

impl Instruction {
    /// Create a new Instruction from a u32. Fails if more than 25 bits
    /// are used.
    pub fn new(instruction: u32) -> Option<Instruction> {
        if instruction.leading_zeros() < 32 - 25 {
            None
        } else {
            Some(Instruction { instruction: instruction })
        }
    }

    /// Creat a new Instruction from a binary string (consisting only of ones
    /// and zeroes). Failes if more than 25 bits (characters) are used.
    pub fn new_from_string(string: &str) -> Option<Instruction> {
        if string.len() > 25 {
            None
        } else {
            u32::from_str_radix(string, 2).ok()
                .map(|instruction| Instruction { instruction: instruction })
        }
    }

    /// Get the instruction as a 25 bit integer (the first 7 most significant
    /// bits of the u32 are always zero)
    pub fn get_instruction(&self) -> u32 {
        self.instruction
    }

    /// MCHFLG
    pub fn should_store_flags(&self) -> bool {
        self.extract_bit(0)
    }

    /// MALUS0-3 (4 bit)
    pub fn get_alu_instruction(&self) -> u8 {
        self.extract_bit_pattern(0b1111, 1)
    }

    /// MALUIB
    pub fn is_alu_input_b_const(&self) -> bool {
        self.extract_bit(5)
    }

    /// MALUIA
    pub fn is_alu_input_a_bus(&self) -> bool {
        self.extract_bit(6)
    }

    /// MRGWE
    pub fn should_write_register(&self) -> bool {
        self.extract_bit(7)
    }

    /// MRGWS
    pub fn should_write_register_b(&self) -> bool {
        self.extract_bit(8)
    }

    /// MRGAB0-2 (3 bit)
    pub fn get_register_address_b(&self) -> usize {
        self.extract_bit_pattern(0b111, 9) as usize
    }

    /// MRGAB0-3 (4 bit)
    pub fn get_constant_input(&self) -> u8 {
        self.extract_bit_pattern(0b1111, 9)
    }

    /// MRGAA0-2 (3 bit)
    pub fn get_register_address_a(&self) -> usize {
        self.extract_bit_pattern(0b111, 13) as usize
    }

    /// BUSEN
    pub fn should_enable_bus(&self) -> bool {
        self.extract_bit(16)
    }

    /// BUSWR
    pub fn should_enable_bus_write(&self) -> bool {
        self.extract_bit(17)
    }

    /// NA0-4 (5 bit)
    pub fn get_next_instruction_address(&self) -> u8 {
        self.extract_bit_pattern(0b11111, 18)
    }

    /// MAC0-1 (2 bit)
    pub fn get_address_control(&self) -> u8 {
        self.extract_bit_pattern(0b11, 23)
    }

    fn extract_bit(&self, position: u8) -> bool {
        return self.instruction & 0b1 << position != 0;
    }

    fn extract_bit_pattern(&self, mask: u8, position: u8) -> u8 {
        return ((self.instruction & (mask as u32) << position) >> position) as u8;
    }
}

impl Cpu {
    /// Create a new cpu with all registers and flags set to zero.
    pub fn new() -> Cpu {
        Cpu {
            registers: [0; 8],
            flags: (false, false, false),
        }
    }

    /// Execute the given instruction on the cpu using the given alu, bus,
    /// input and output. Returns the address of the next instruction.
    pub fn execute_instruction<A>(&mut self, inst: Instruction, alu: A, bus: &mut [u8; 252],
        input: &[u8; 4], output: &mut [u8; 2]) -> Result<u8, &'static str>
        where A: Fn(u8, u8, u8, bool) -> (u8, (bool, bool, bool)) {
        let a;
        let b;

        // Determine alu input a (bus or register)
        if inst.is_alu_input_a_bus() {
            if ! inst.should_enable_bus() {
                return Err("Cannot read from disabled bus");
            } else if inst.should_enable_bus_write() {
                return Err("Cannot read from bus while it is in write mode");
            }

            let address = inst.get_register_address_a();
            if address >= 0xFC { // FC - FF are input registers
                a = input[(address - 0xFC)]
            } else {
                a = bus[address];
            }
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
        let (result, flags) = alu(inst.get_alu_instruction(), a, b, self.flags.0);

        // Write result to registers
        if inst.should_write_register() {
            if inst.should_write_register_b() {
                self.registers[inst.get_register_address_b()] = result;
            } else {
                self.registers[inst.get_register_address_a()] = result;
            }
        }

        // Write results to the bus
        if inst.should_enable_bus() && inst.should_enable_bus_write() {
            let address = self.registers[inst.get_register_address_a()] as usize;

            if address == 0xFC && address == 0xFD {
                return Err("Cannot write into input register");
            }

            if address >= 0xFE {
                output[address - 0xFE] = result;
            } else {
                bus[address] = result;
            }
        }

        Ok(Cpu::calculate_next_instruction_address(inst, flags, self.flags.0))
    }

    /// Calculate the next instruction address based on the current instruction
    /// and the flags.
    fn calculate_next_instruction_address(inst: Instruction,
        flags: (bool, bool, bool), stored_carry: bool) -> u8 {
        let next_address = inst.get_next_instruction_address();

        match inst.get_address_control() << 1 | (next_address & 0b00001) {
            0b000 | 0b001 => {
                next_address
            }
            0b010 => {
                next_address | 0b00001
            }
            0b011 => {
                next_address | stored_carry as u8
            }
            0b100 => {
                next_address | flags.0 as u8
            }
            0b101 => {
                next_address | flags.2 as u8
            }
            0b110 => {
                next_address | flags.1 as u8
            }
            0b111 => {
                next_address & 0b11110
            }
            _ => {
                panic!("Invlid address control")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    mod instruction {
        use super::super::*;

        #[test]
        #[should_panic]
        fn from_long_integer() {
            Instruction::new(0b1000000000000000000000000_0).unwrap();
        }

        #[test]
        fn extract_fields () {
            // load constant FC into register 0
            let i1 = Instruction::new(0b00_00001_00_000_1100_01_01_0001_0).unwrap();
            assert_eq!(i1.should_store_flags(), false);
            assert_eq!(i1.get_alu_instruction(), 0b0001);
            assert_eq!(i1.is_alu_input_b_const(), true);
            assert_eq!(i1.is_alu_input_a_bus(), false);
            assert_eq!(i1.should_write_register(), true);
            assert_eq!(i1.should_write_register_b(), false);
            assert_eq!(i1.get_constant_input(), 0b1100);
            assert_eq!(i1.get_register_address_b(), 0b100);
            assert_eq!(i1.get_register_address_a(), 0b000);
            assert_eq!(i1.should_enable_bus(), false);
            assert_eq!(i1.should_enable_bus_write(), false);
            assert_eq!(i1.get_next_instruction_address(), 0b00001);
            assert_eq!(i1.get_address_control(), 0b00);

            // load from memory location FC (register 0) into register 2
            let i1 = Instruction::new(0b00_00010_01_000_0010_11_10_0000_0).unwrap();
            assert_eq!(i1.should_store_flags(), false);
            assert_eq!(i1.get_alu_instruction(), 0b0000);
            assert_eq!(i1.is_alu_input_b_const(), false);
            assert_eq!(i1.is_alu_input_a_bus(), true);
            assert_eq!(i1.should_write_register(), true);
            assert_eq!(i1.should_write_register_b(), true);
            assert_eq!(i1.get_constant_input(), 0b0010);
            assert_eq!(i1.get_register_address_b(), 0b010);
            assert_eq!(i1.get_register_address_a(), 0b000);
            assert_eq!(i1.should_enable_bus(), true);
            assert_eq!(i1.should_enable_bus_write(), false);
            assert_eq!(i1.get_next_instruction_address(), 0b00010);
            assert_eq!(i1.get_address_control(), 0b00);
        }

        #[test]
        fn from_string() {
            let i1a = Instruction::new(0b00_00001_00_000_1100_01_01_0001_0).unwrap();
            let i2a = Instruction::new(0b00_00010_01_000_0010_11_10_0000_0).unwrap();
            let i3a = Instruction::new(0b11_11111_11_111_1111_11_11_1111_1).unwrap();
            let i1b = Instruction::new_from_string("0000001000001100010100010").unwrap();
            let i2b = Instruction::new_from_string("0000010010000010111000000").unwrap();
            let i3b = Instruction::new_from_string("1111111111111111111111111").unwrap();

            assert_eq!(i1a.get_instruction(), i1b.get_instruction());
            assert_eq!(i2a.get_instruction(), i2b.get_instruction());
            assert_eq!(i3a.get_instruction(), i3b.get_instruction());
        }

        #[test]
        #[should_panic]
        fn from_long_string() {
            Instruction::new_from_string("11111111111111111111111110").unwrap();
        }
    }
}
