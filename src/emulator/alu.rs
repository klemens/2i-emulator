//! The 2i 8 bit alu.
//!
//! This module contains the alu used in the 2i.

/// Alu of the 2i.
pub struct Alu;

impl Alu {
    /// Execute an instruction with two operands on the alu.
    ///
    /// Returns the result and the resulting flags. Higher instructions than
    /// 1111 == 15 will result in a panic.
    pub fn calculate(instruction: u8, a: u8, b: u8, carry: bool) -> (u8, Flags) {
        let (result, carry) = match instruction {
            0b0000 => (a, false),
            0b0001 => (b, false),
            0b0010 => (!(a | b), false),
            0b0011 => (0, false),
            0b0100 => a.overflowing_add(b),
            0b0101 => { // inverted carry
                let tmp1 = a.overflowing_add(b);
                let tmp2 = tmp1.0.overflowing_add(1);
                (tmp2.0, !(tmp1.1 | tmp2.1))
            }
            0b0110 => {
                let tmp1 = a.overflowing_add(b);
                let tmp2 = tmp1.0.overflowing_add(if carry {1} else {0});
                (tmp2.0, tmp1.1 | tmp2.1)
            }
            0b0111 => { // inverted carry
                let tmp1 = a.overflowing_add(b);
                let tmp2 = tmp1.0.overflowing_add(if carry {0} else {1});
                (tmp2.0, !(tmp1.1 | tmp2.1))
            }
            0b1000 => (a >> 1, a & 0b00000001 != 0),
            0b1001 => (a.rotate_right(1), a & 0b00000001 != 0),
            0b1010 => (a >> 1 | (carry as u8) << 7, a & 0b00000001 != 0),
            0b1011 => (a >> 1 | (a & 0b10000000), a & 0b00000001 != 0),
            0b1100 => (0, false),
            0b1101 => (0, true),
            0b1110 => (0, carry),
            0b1111 => (0, !carry),
            _ => panic!("Invalid alu instruction {}", instruction),
        };

        let negative = result & 0b10000000 != 0; // two's complement
        let zero = result == 0;

        return (result, Flags::new(carry, negative, zero));
    }
}

/// Flags of the 2i.
///
/// Represents the flags used by the alu to describe its result. Can be used
/// for conditional jumps and as further input to the alu in case of the carry.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Flags {
    carry: bool,
    negative: bool,
    zero: bool,
}

impl Flags {
    pub fn new(carry: bool, negative: bool, zero: bool) -> Flags {
        Flags { carry: carry, negative: negative, zero: zero }
    }

    pub fn carry(&self) -> bool {
        self.carry
    }
    pub fn negative(&self) -> bool {
        self.negative
    }
    pub fn zero(&self) -> bool {
        self.zero
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logic() {
        let a = 0b11010100;
        let b = 0b00101101;

        // pass through a
        assert_eq!(Alu::calculate(0b0000, a, b, false), (a, Flags::new(false,  true, false)));
        // pass through b
        assert_eq!(Alu::calculate(0b0001, a, b, false), (b, Flags::new(false, false, false)));
        // return 0
        assert_eq!(Alu::calculate(0b0011, a, b, false), (0, Flags::new(false, false,  true)));

        // nor
        assert_eq!(Alu::calculate(0b0010, a, b, false), (0b00000010, Flags::new(false, false, false)));
        // invert (using nor)
        assert_eq!(Alu::calculate(0b0010, a, a, false), (0b00101011, Flags::new(false, false, false)));
        assert_eq!(Alu::calculate(0b0010, b, b, false), (0b11010010, Flags::new(false,  true, false)));
    }

    #[test]
    fn addition() {
        // add
        assert_eq!(Alu::calculate(0b0100,  0,   0, false), ( 0, Flags::new(false, false, true)));
        assert_eq!(Alu::calculate(0b0100,  0,  19, false), (19, Flags::new(false, false, false)));
        assert_eq!(Alu::calculate(0b0100, 47,   0, false), (47, Flags::new(false, false, false)));
        assert_eq!(Alu::calculate(0b0100, 47,  19, false), (66, Flags::new(false, false, false)));
        assert_eq!(Alu::calculate(0b0100, 47, 236, false), (27, Flags::new( true, false, false)));

        // add1 (inverts carry)
        assert_eq!(Alu::calculate(0b0101,  0,   0, false), ( 1, Flags::new( true, false, false)));
        assert_eq!(Alu::calculate(0b0101,  0,  19, false), (20, Flags::new( true, false, false)));
        assert_eq!(Alu::calculate(0b0101, 47,   0, false), (48, Flags::new( true, false, false)));
        assert_eq!(Alu::calculate(0b0101, 47,  19, false), (67, Flags::new( true, false, false)));
        assert_eq!(Alu::calculate(0b0101, 47, 236, false), (28, Flags::new(false, false, false)));

        // addc
        assert_eq!(Alu::calculate(0b0110, 47,  19, false), (66, Flags::new(false, false, false)));
        assert_eq!(Alu::calculate(0b0110, 47,  19,  true), (67, Flags::new(false, false, false)));
        assert_eq!(Alu::calculate(0b0110, 47, 236, false), (27, Flags::new( true, false, false)));
        assert_eq!(Alu::calculate(0b0110, 47, 236,  true), (28, Flags::new( true, false, false)));

        // addci (inverts carry)
        assert_eq!(Alu::calculate(0b0111, 47,  19, false), (67, Flags::new( true, false, false)));
        assert_eq!(Alu::calculate(0b0111, 47,  19,  true), (66, Flags::new( true, false, false)));
        assert_eq!(Alu::calculate(0b0111, 47, 236, false), (28, Flags::new(false, false, false)));
        assert_eq!(Alu::calculate(0b0111, 47, 236,  true), (27, Flags::new(false, false, false)));
    }

    #[test]
    fn shifts() {
        let a = 0b11010100;
        let b = 0b00101101;

        // left shift (using addition)
        assert_eq!(Alu::calculate(0b0100, a, a, false), (0b10101000, Flags::new( true,  true, false)));
        assert_eq!(Alu::calculate(0b0100, b, b, false), (0b01011010, Flags::new(false, false, false)));

        // logic right shift
        assert_eq!(Alu::calculate(0b1000, a, 0, false), (0b01101010, Flags::new(false, false, false)));
        assert_eq!(Alu::calculate(0b1000, b, 0, false), (0b00010110, Flags::new( true, false, false)));

        // algebraic right shift
        assert_eq!(Alu::calculate(0b1011, a, 0, false), (0b11101010, Flags::new(false,  true, false)));
        assert_eq!(Alu::calculate(0b1011, b, 0, false), (0b00010110, Flags::new( true, false, false)));

        // right rotation
        assert_eq!(Alu::calculate(0b1001, a, 0, false), (0b01101010, Flags::new(false, false, false)));
        assert_eq!(Alu::calculate(0b1001, b, 0, false), (0b10010110, Flags::new( true,  true, false)));

        // right carry rotation
        assert_eq!(Alu::calculate(0b1010, a, 0, false), (0b01101010, Flags::new(false, false, false)));
        assert_eq!(Alu::calculate(0b1010, a, 0,  true), (0b11101010, Flags::new(false,  true, false)));
        assert_eq!(Alu::calculate(0b1010, b, 0, false), (0b00010110, Flags::new( true, false, false)));
        assert_eq!(Alu::calculate(0b1010, b, 0,  true), (0b10010110, Flags::new( true,  true, false)));
    }

    #[test]
    fn flags() {
        // clear carry
        assert_eq!(Alu::calculate(0b1100, 0, 0, false), (0, Flags::new(false, false, true)));
        assert_eq!(Alu::calculate(0b1100, 0, 0,  true), (0, Flags::new(false, false, true)));

        // set carry
        assert_eq!(Alu::calculate(0b1101, 0, 0, false), (0, Flags::new( true, false, true)));
        assert_eq!(Alu::calculate(0b1101, 0, 0,  true), (0, Flags::new( true, false, true)));

        // get carry (equal to 0b0011)
        assert_eq!(Alu::calculate(0b1110, 0, 0, false), (0, Flags::new(false, false, true)));
        assert_eq!(Alu::calculate(0b1110, 0, 0,  true), (0, Flags::new( true, false, true)));

        // invert carry (equal to 0b0011)
        assert_eq!(Alu::calculate(0b1111, 0, 0, false), (0, Flags::new( true, false, true)));
        assert_eq!(Alu::calculate(0b1111, 0, 0,  true), (0, Flags::new(false, false, true)));
    }

    #[test]
    #[should_panic(expected = "Invalid alu instruction")]
    fn invalid_instruction() {
        Alu::calculate(0b10000, 0, 0, false);
    }
}
