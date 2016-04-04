//! The 2i cpu.
//!
//! This module contains the cpu used in the 2i.

/// Instruction of the 2i.
///
/// Represents a 25 bit wide instruction that the 2i uses. Provides some
/// conveniance methods to extract all different parts.
pub struct Instruction {
    instruction: u32
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

    /// MRGAB0-3 (4 bit)
    pub fn get_register_address_b(&self) -> u8 {
        self.extract_bit_pattern(0b1111, 9)
    }

    /// MRGAA0-2 (3 bit)
    pub fn get_register_address_a(&self) -> u8 {
        self.extract_bit_pattern(0b111, 13)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction() {
        // load constant FC into register 0
        let i1 = Instruction::new(0b00_00001_00_000_1100_01_01_0001_0).unwrap();
        assert_eq!(i1.should_store_flags(), false);
        assert_eq!(i1.get_alu_instruction(), 0b0001);
        assert_eq!(i1.is_alu_input_b_const(), true);
        assert_eq!(i1.is_alu_input_a_bus(), false);
        assert_eq!(i1.should_write_register(), true);
        assert_eq!(i1.should_write_register_b(), false);
        assert_eq!(i1.get_register_address_b(), 0b1100);
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
        assert_eq!(i1.get_register_address_b(), 0b0010);
        assert_eq!(i1.get_register_address_a(), 0b000);
        assert_eq!(i1.should_enable_bus(), true);
        assert_eq!(i1.should_enable_bus_write(), false);
        assert_eq!(i1.get_next_instruction_address(), 0b00010);
        assert_eq!(i1.get_address_control(), 0b00);
    }
}
