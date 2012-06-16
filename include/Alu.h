#ifndef _MINIRECHNER2I_ALU_H
#define _MINIRECHNER2I_ALU_H

#include <bitset>

namespace Minirechner2i {

using std::bitset;

/**
 * 8 bit Alu with an adder, a logical nor and a right shifter.
 * All bitfields big endian.
 *
 * Function table:
 * <pre>
 *     ______________A_=_B___Result______________________C_____N_Z
 *     0000 | A      -       F = A                       0     * *
 *     0001 | B      -       F = B                       0     * *
 *     0010 | NOR    COM     F = A NOR B                 0     * *
 *     0011 | 0      -       F = 0                       0     0 1
 *     0100 | ADD    LSL     F = A + B                   Ca    * *
 *     0101 | ADD+1  (SL1)   F = A + B + 1               ~Ca   * *
 *     0110 | ADC    RLC     F = A + B + Cin             Ca    * *
 *     0111 | ADCI   (RLCI)  F = A + B + ~Cin            ~Ca   * *
 *     1000 | LSR    -       F(n) = A(n+1), F(7) = 0     A(0)  * *
 *     1001 | RR     -       F(n) = A(n+1), F(7) = A(0)  A(0)  * *
 *     1010 | RRC    -       F(n) = A(n+1), F(7) = Cin   A(0)  * *
 *     1011 | ASR    -       F(n) = A(n+1), F(7) = A(7)  A(0)  * *
 *     1100 | CLC    -       F = 0                       0     0 1
 *     1101 | SETC   -       F = 0                       1     0 1
 *     1110 | (LETC) -       F = 0                       Cin   0 1
 *     1111 | INVC   -       F = 0                       ~Cin  0 1
 *
 *      C = carry (overflow)
 *      N = output is negative (last bit set -> two's complement)
 *      Z = output is 0
 */
class Alu {
    public:
        /**
         * Applies the given function to the arguments a and b (see class description)
         *
         * falgs[0] should countain the carry in bit and will be filled as follows:
         * flags[0] = carry out
         * flags[1] = result is negative (last bit 1)
         * falgs[2] = result is null (all bits 0)
         */
        bitset<8> calculate(bitset<4> function, bitset<8> a, bitset<8> b, bitset<3> & flags);
};

}

#endif
