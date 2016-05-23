//! The 2i instruction

use std::fmt;

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
        let c = self.extract_bit_pattern(0b1111, 9);

        if c & 0b1000 != 0 {
            // Set bits 4-7 to one if bit 3 is set
            c | 0b11110000
        } else {
            c
        }
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

    /// Create a textual representation of the instruction
    ///
    /// Optionally, the address of the next instruction after this one can be
    /// passed to simplify the resulting string in many cases.
    ///
    /// # Examples
    ///
    /// ```
    /// use emulator::Instruction;
    ///
    /// let inst = Instruction::new(0b00_00000_00_000_0110_01_01_0100_0).unwrap();
    /// assert_eq!("R0 = R0 + 6", &inst.to_text_paraphrase(Some(0)));
    /// ```
    pub fn to_text_paraphrase(&self, next_address: Option<usize>) -> String {
        // Determine input a
        let a = if self.is_alu_input_a_bus() {
            format!("(R{})", self.get_register_address_a())
        } else {
            format!("R{}", self.get_register_address_a())
        };

        // Determine input b
        let b = if self.is_alu_input_b_const() {
            format!("{:X}", self.get_constant_input())
        } else {
            format!("R{}", self.get_register_address_b())
        };

        // Determine alu function
        let result = match self.get_alu_instruction() {
            0b0000 => a,
            0b0001 => b,
            0b0010 => format!("{} ⊽ {}", a, b),
            0b0011 => "0".to_string(),
            0b0100 => format!("{} + {}", a, b),
            0b0101 => format!("{} + {} + 1", a, b),
            0b0110 => format!("{} + {} + C", a, b),
            0b0111 => format!("{} + {} + C̅", a, b),
            0b1000 => format!("{} >> 1", a),
            0b1001 => format!("{} ⤿> 1", a),
            0b1010 => format!("{} C> 1", a),
            0b1011 => format!("{} ?> 1", a),
            0b1100 => "0".to_string(),
            0b1101 => "0".to_string(),
            0b1110 => "0".to_string(),
            0b1111 => "0".to_string(),
            i => panic!("Invalid instruction {}", i),
        };

        // Determine output
        let format_register = || {
            format!("R{}", if self.should_write_register_b() {
                self.get_register_address_b()
            } else {
                self.get_register_address_a()
            })
        };
        let output = if self.is_bus_enabled() && self.is_bus_writable() {
            if self.should_write_register() {
                format!("(R{}), {} = ", self.get_register_address_a(), format_register())
            } else {
                format!("(R{}) = ", self.get_register_address_a())
            }
        } else if self.should_write_register() {
            format!("{} = ", format_register())
        } else {
            "TEST ".to_string()
        };

        // Determine address control and next address
        let address_control = if self.get_address_control() == 0 &&
            next_address == Some(self.get_next_instruction_address() as usize) {
            String::new()
        } else {
            let next_address = self.get_next_instruction_address();
            let next_address_base = next_address >> 1; // Cut off last bit

            match self.get_address_control() << 1 | (next_address & 0b00001) {
                0b000 | 0b001 => format!("; JMP {:05b}", next_address),
                0b010 => format!("; JMP {:04b}1", next_address_base),
                0b011 => format!("; CF {:04b}C", next_address_base),
                0b100 => format!("; CO {:04b}C", next_address_base),
                0b101 => format!("; ZO {:04b}Z", next_address_base),
                0b110 => format!("; NO {:04b}N", next_address_base),
                0b111 => format!("; JMP {:04b}0", next_address_base),
                _ => panic!("Invalid address control"),
            }
        };

        // Determine flag storage
        let change_flags = if self.should_store_flags() {
            "; CHFL".to_string()
        } else {
            String::new()
        };

        if self.instruction & 0b1100000111111111111111110 == 0 {
            // NOP if everything except NA and CHFL is zero
            format!("NOP{}{}", address_control, change_flags)
        } else {
            format!("{}{}{}{}", output, result, address_control, change_flags)
        }
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Instruction {{ {:025b} }}", self.instruction)
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
        assert_eq!(i1.get_constant_input(), 0b11111100);
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
        assert_eq!(i1.get_constant_input(), 0b00000010);
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

    #[test]
    fn to_string() {
        let testcases = [
            (0b00_00000_00_000_0000_00_00_0000_0, "NOP", Some(0)),
            (0b00_00000_00_000_0000_00_00_0000_0, "NOP; JMP 00000", None),
            (0b00_00000_00_000_0000_00_00_0001_0, "TEST R0", Some(0)),
            (0b00_00000_00_000_0000_00_00_0000_1, "NOP; CHFL", Some(0)),
            (0b00_00000_00_000_1100_01_01_0001_0, "R0 = FC", Some(0)),
            (0b00_00000_00_000_1100_01_01_0001_0, "R0 = FC; JMP 00000", None),
            (0b00_00000_00_000_1100_01_01_0001_0, "R0 = FC; JMP 00000", Some(1)),
            (0b00_00000_01_000_0000_01_10_0000_0, "R0 = (R0)", Some(0)),
            (0b00_00000_11_001_0010_00_00_0001_0, "(R1) = R2", Some(0)),
            (0b00_00000_11_001_0011_01_01_0001_0, "(R1), R1 = 3", Some(0)),
            (0b01_00010_00_000_0000_00_00_0000_0, "TEST R0; JMP 00011", None),
            (0b01_00101_00_000_0000_00_00_0000_0, "TEST R0; CF 0010C", None),
            (0b10_00110_00_000_0000_00_00_0000_0, "TEST R0; CO 0011C", None),
            (0b10_01001_00_000_0000_00_00_0000_0, "TEST R0; ZO 0100Z", None),
            (0b11_01010_00_000_0000_00_00_0000_0, "TEST R0; NO 0101N", None),
            (0b11_01101_00_000_0000_00_00_0000_0, "TEST R0; JMP 01100", None),
            (0b00_00000_00_000_1111_01_01_0010_0, "R0 = R0 ⊽ FF", Some(0)),
            (0b00_00000_00_010_0000_01_00_0011_0, "R2 = 0", Some(0)),
            (0b00_00000_00_000_1111_01_01_0100_0, "R0 = R0 + FF", Some(0)),
            (0b00_00000_00_000_1111_01_01_0101_0, "R0 = R0 + FF + 1", Some(0)),
            (0b00_00000_00_000_1111_01_01_0110_0, "R0 = R0 + FF + C", Some(0)),
            (0b00_00000_00_000_1111_01_01_0111_0, "R0 = R0 + FF + C̅", Some(0)),
            (0b00_00000_00_000_1111_01_01_1000_0, "R0 = R0 >> 1", Some(0)),
            (0b00_00000_00_000_1111_01_01_1001_0, "R0 = R0 ⤿> 1", Some(0)),
            (0b00_00000_00_000_1111_01_01_1010_0, "R0 = R0 C> 1", Some(0)),
            (0b00_00000_00_000_1111_01_01_1011_0, "R0 = R0 ?> 1", Some(0)),
        ];

        for &(i, s, na) in testcases.iter() {
            assert_eq!(Instruction::new(i).unwrap().to_text_paraphrase(na), s.to_string());
        }
    }
}
