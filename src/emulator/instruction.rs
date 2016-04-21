//! The 2i instruction

use super::{Error, Result};

/// Instruction of the 2i.
///
/// Represents a 25 bit wide instruction that the 2i uses. Provides some
/// conveniance methods to extract all different parts.
#[derive(Copy, Clone)]
pub struct Instruction {
    instruction: u32,
}

impl Instruction {
    /// Create a new Instruction from a u32. Fails if more than 25 bits
    /// are used.
    pub fn new(instruction: u32) -> Result<Instruction> {
        if instruction.leading_zeros() < 32 - 25 {
            Err(Error::Instruction("Given u32 too large (more then 25 bit)"))
        } else {
            Ok(Instruction { instruction: instruction })
        }
    }

    /// Creat a new Instruction from a binary string (consisting only of ones
    /// and zeroes). Failes if more than 25 bits (characters) are used.
    pub fn new_from_string(string: &str) -> Result<Instruction> {
        if string.len() > 25 {
            Err(Error::Instruction("Given str too large (more then 25 bit)"))
        } else {
            u32::from_str_radix(string, 2)
                .map_err(|_| Error::Instruction("Error parsing the given str"))
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
    pub fn is_bus_enabled(&self) -> bool {
        self.extract_bit(16)
    }

    /// BUSWR
    pub fn is_bus_writable(&self) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn from_long_integer() {
        Instruction::new(0b1000000000000000000000000_0).unwrap();
    }

    #[test]
    fn extract_fields () {
        // Load constant FC into register 0
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
        assert_eq!(i1.is_bus_enabled(), false);
        assert_eq!(i1.is_bus_writable(), false);
        assert_eq!(i1.get_next_instruction_address(), 0b00001);
        assert_eq!(i1.get_address_control(), 0b00);

        // Load from memory location FC (register 0) into register 2
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
        assert_eq!(i1.is_bus_enabled(), true);
        assert_eq!(i1.is_bus_writable(), false);
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

    #[test]
    #[should_panic]
    fn from_invalid_string() {
        Instruction::new_from_string("00a0010010000010111000000").unwrap();
    }
}
