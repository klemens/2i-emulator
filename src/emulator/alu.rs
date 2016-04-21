//! The 2i 8 bit alu.
//!
//! This module contains the alu used in the 2i.

/// Execute an instruction with two operands on the alu.
///
/// Implements an adder, a logical nor and a right shifter using the following
/// instruction table:
///
/// ```text
/// ______________A_=_B___Result______________________C_____N_Z
/// 0000 | A      -       F = A                       0     * *
/// 0001 | B      -       F = B                       0     * *
/// 0010 | NOR    COM     F = A NOR B                 0     * *
/// 0011 | 0      -       F = 0                       0     0 1
/// 0100 | ADD    LSL     F = A + B                   Ca    * *
/// 0101 | ADD+1  (SL1)   F = A + B + 1               ~Ca   * *
/// 0110 | ADC    RLC     F = A + B + Cin             Ca    * *
/// 0111 | ADCI   (RLCI)  F = A + B + ~Cin            ~Ca   * *
/// 1000 | LSR    -       F(n) = A(n+1), F(7) = 0     A(0)  * *
/// 1001 | RR     -       F(n) = A(n+1), F(7) = A(0)  A(0)  * *
/// 1010 | RRC    -       F(n) = A(n+1), F(7) = Cin   A(0)  * *
/// 1011 | ASR    -       F(n) = A(n+1), F(7) = A(7)  A(0)  * *
/// 1100 | CLC    -       F = 0                       0     0 1
/// 1101 | SETC   -       F = 0                       1     0 1
/// 1110 | (LETC) -       F = 0                       Cin   0 1
/// 1111 | INVC   -       F = 0                       ~Cin  0 1
///
/// C = carry (overflow)
/// N = output is negative (two's complement: msb is 1)
/// Z = output is 0
/// ```
///
/// Returns the result and the flags carry, negative and zero as tuple. Higher
/// instructions than 1111 == 15 will result in a panic.
pub fn calculate(instruction: u8, a: u8, b: u8, mut carry: bool) -> (u8, Flags) {
    let f; // result

    match instruction {
        0b0000 => { // f = a
            f = a;
            carry = false;
        }
        0b0001 => { // f = b
            f = b;
            carry = false;
        }
        0b0010 => { // f = a NOR b
            f = ! (a | b);
            carry = false;
        }
        0b0011 => { // f = 0
            f = 0;
            carry = false;
        }
        0b0100 => { // f = a + b
            let tmp = a.overflowing_add(b);
            f = tmp.0;
            carry = tmp.1;
        }
        0b0101 => { // f = a + b + 1, inverted carry
            let tmp1 = a.overflowing_add(b);
            let tmp2 = tmp1.0.overflowing_add(1);
            f = tmp2.0;
            carry = ! (tmp1.1 | tmp2.1);
        }
        0b0110 => { // f = a + b + carry
            let tmp1 = a.overflowing_add(b);
            let tmp2 = tmp1.0.overflowing_add(if carry {1} else {0});
            f = tmp2.0;
            carry = tmp1.1 | tmp2.1;
        }
        0b0111 => { // f = a + b + !carry, carry inverted
            let tmp1 = a.overflowing_add(b);
            let tmp2 = tmp1.0.overflowing_add(if carry {0} else {1});
            f = tmp2.0;
            carry = ! (tmp1.1 | tmp2.1);
        }
        0b1000 => { // f = a >> 1, carry = a[0] (bit shifted out)
            f = a >> 1;
            carry = a & 0b00000001 != 0;
        }
        0b1001 => { // f = a >>(rotate) 1, carry = a[0] (bit shifted out)
            f = a.rotate_right(1);
            carry = a & 0b00000001 != 0;
        }
        0b1010 => { // f = a >> 1, f[7] = carry, carry = a[0] (bit shifted out)
            f = a >> 1 | (carry as u8) << 7;
            carry = a & 0b00000001 != 0;
        }
        0b1011 => { // f = a >> 1, f[7] = a[7], carry = a[0] (bit shifted out)
            f = a >> 1 | (a & 0b10000000);
            carry = a & 0b00000001 != 0;
        }
        0b1100 => { // f = 0, clear carry
            f = 0;
            carry = false;
        }
        0b1101 => { // f = 0, set carry
            f = 0;
            carry = true;
        }
        0b1110 => { // f = 0, let carry
            f = 0;
            carry = carry;
        }
        0b1111 => { // f = 0, invert carry
            f = 0;
            carry = ! carry;
        }
        _ => {
            panic!("Invalid instruction {}", instruction);
        }
    }

    let negative = f & 0b10000000 != 0; // two's complement
    let zero = f == 0;

    return (f, Flags::new(carry, negative, zero));
}

/// Flags of the 2i
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
    use super::calculate as calc;
    use super::Flags;

    #[test]
    fn logic() {
        let a = 0b11010100;
        let b = 0b00101101;

        // pass through a
        assert_eq!(calc(0b0000, a, b, false), (a, Flags::new(false,  true, false)));
        // pass through b
        assert_eq!(calc(0b0001, a, b, false), (b, Flags::new(false, false, false)));
        // return 0
        assert_eq!(calc(0b0011, a, b, false), (0, Flags::new(false, false,  true)));

        // nor
        assert_eq!(calc(0b0010, a, b, false), (0b00000010, Flags::new(false, false, false)));
        // invert (using nor)
        assert_eq!(calc(0b0010, a, a, false), (0b00101011, Flags::new(false, false, false)));
        assert_eq!(calc(0b0010, b, b, false), (0b11010010, Flags::new(false,  true, false)));
    }

    #[test]
    fn addition() {
        // add
        assert_eq!(calc(0b0100,  0,   0, false), ( 0, Flags::new(false, false, true)));
        assert_eq!(calc(0b0100,  0,  19, false), (19, Flags::new(false, false, false)));
        assert_eq!(calc(0b0100, 47,   0, false), (47, Flags::new(false, false, false)));
        assert_eq!(calc(0b0100, 47,  19, false), (66, Flags::new(false, false, false)));
        assert_eq!(calc(0b0100, 47, 236, false), (27, Flags::new( true, false, false)));

        // add1 (inverts carry)
        assert_eq!(calc(0b0101,  0,   0, false), ( 1, Flags::new( true, false, false)));
        assert_eq!(calc(0b0101,  0,  19, false), (20, Flags::new( true, false, false)));
        assert_eq!(calc(0b0101, 47,   0, false), (48, Flags::new( true, false, false)));
        assert_eq!(calc(0b0101, 47,  19, false), (67, Flags::new( true, false, false)));
        assert_eq!(calc(0b0101, 47, 236, false), (28, Flags::new(false, false, false)));

        // addc
        assert_eq!(calc(0b0110, 47,  19, false), (66, Flags::new(false, false, false)));
        assert_eq!(calc(0b0110, 47,  19,  true), (67, Flags::new(false, false, false)));
        assert_eq!(calc(0b0110, 47, 236, false), (27, Flags::new( true, false, false)));
        assert_eq!(calc(0b0110, 47, 236,  true), (28, Flags::new( true, false, false)));

        // addci (inverts carry)
        assert_eq!(calc(0b0111, 47,  19, false), (67, Flags::new( true, false, false)));
        assert_eq!(calc(0b0111, 47,  19,  true), (66, Flags::new( true, false, false)));
        assert_eq!(calc(0b0111, 47, 236, false), (28, Flags::new(false, false, false)));
        assert_eq!(calc(0b0111, 47, 236,  true), (27, Flags::new(false, false, false)));
    }

    #[test]
    fn shifts() {
        let a = 0b11010100;
        let b = 0b00101101;

        // left shift (using addition)
        assert_eq!(calc(0b0100, a, a, false), (0b10101000, Flags::new( true,  true, false)));
        assert_eq!(calc(0b0100, b, b, false), (0b01011010, Flags::new(false, false, false)));

        // logic right shift
        assert_eq!(calc(0b1000, a, 0, false), (0b01101010, Flags::new(false, false, false)));
        assert_eq!(calc(0b1000, b, 0, false), (0b00010110, Flags::new( true, false, false)));

        // algebraic right shift
        assert_eq!(calc(0b1011, a, 0, false), (0b11101010, Flags::new(false,  true, false)));
        assert_eq!(calc(0b1011, b, 0, false), (0b00010110, Flags::new( true, false, false)));

        // right rotation
        assert_eq!(calc(0b1001, a, 0, false), (0b01101010, Flags::new(false, false, false)));
        assert_eq!(calc(0b1001, b, 0, false), (0b10010110, Flags::new( true,  true, false)));

        // right carry rotation
        assert_eq!(calc(0b1010, a, 0, false), (0b01101010, Flags::new(false, false, false)));
        assert_eq!(calc(0b1010, a, 0,  true), (0b11101010, Flags::new(false,  true, false)));
        assert_eq!(calc(0b1010, b, 0, false), (0b00010110, Flags::new( true, false, false)));
        assert_eq!(calc(0b1010, b, 0,  true), (0b10010110, Flags::new( true,  true, false)));
    }

    #[test]
    fn flags() {
        // clear carry
        assert_eq!(calc(0b1100, 0, 0, false), (0, Flags::new(false, false, true)));
        assert_eq!(calc(0b1100, 0, 0,  true), (0, Flags::new(false, false, true)));

        // set carry
        assert_eq!(calc(0b1101, 0, 0, false), (0, Flags::new( true, false, true)));
        assert_eq!(calc(0b1101, 0, 0,  true), (0, Flags::new( true, false, true)));

        // get carry (equal to 0b0011)
        assert_eq!(calc(0b1110, 0, 0, false), (0, Flags::new(false, false, true)));
        assert_eq!(calc(0b1110, 0, 0,  true), (0, Flags::new( true, false, true)));

        // invert carry (equal to 0b0011)
        assert_eq!(calc(0b1111, 0, 0, false), (0, Flags::new( true, false, true)));
        assert_eq!(calc(0b1111, 0, 0,  true), (0, Flags::new(false, false, true)));
    }

    #[test]
    #[should_panic(expected = "Invalid instruction")]
    fn invalid_instruction() {
        calc(0b10000, 0, 0, false);
    }
}
