#include "Alu.h"

using std::bitset;

bitset<8> Minirechner2i::Alu::calculate(bitset<4> function, bitset<8> a,
                                        bitset<8> b, bitset<3> & flags) {
    bitset<8> f; //the result, initialized with 00000000
    unsigned long aLong = a.to_ulong(), bLong = b.to_ulong(), tmp;

    switch(function.to_ulong()) {
        case 0: // F = A
            f = a;
            flags[0] = false;
            break;
        case 1: // F = B
            f = b;
            flags[0] = false;
            break;
        case 2: // F = A NOR B
            f = ~(a | b);
            flags[0] = false;
            break;
        case 3: // F = 0
            f = ~(a | b);
            flags[0] = false;
            break;
        case 4: // F = A + B
            tmp = aLong + bLong;
            f = bitset<8>(tmp);
            flags[0] = tmp > 255; // get the overflow bit
            break;
        case 5: // F = A + B + 1, Ca inverted
            tmp = aLong + bLong + 1;
            f = bitset<8>(tmp);
            flags[0] = ~(tmp > 255); // get the overflow bit
            break;
        case 6: // F = A + B + Cin
            tmp = aLong + bLong + (flags[0] ? 1 : 0);
            f = bitset<8>(tmp);
            flags[0] = tmp > 255; // get the overflow bit
            break;
        case 7: // F = A + B + ~Cin, Ca inverted
            tmp = aLong + bLong + (flags[0] ? 0 : 1);
            f = bitset<8>(tmp);
            flags[0] = ~(tmp > 255); // get the overflow bit
            break;
        case 8: // F(n) = A(n+1), F(7) = 0
            f = a>>1;
            f[7] = false;
            flags[0] = a[0];
            break;
        case 9: // F(n) = A(n+1), F(7) = A(0)
            f = a>>1;
            f[7] = a[0];
            flags[0] = a[0];
            break;
        case 10: // F(n) = A(n+1), F(7) = Cin
            f = a>>1;
            f[7] = flags[0];
            flags[0] = a[0];
            break;
        case 11: // F(n) = A(n+1), F(7) = A(7)
            f = a>>1;
            f[7] = a[7];
            flags[0] = a[0];
            break;
        case 12: // F = 0, clear carry
            flags[0] = false;
            break;
        case 13: // F = 0, set carry
            flags[0] = true;
            break;
        case 14: // F = 0, let carry
            // Cin is already in flags[0]
            break;
        case 15: // F = 0, invert carry
            flags[0] = ~flags[0];
    }

    flags[1] = f[7]; // is negative (as two's complement)
    flags[2] = f.none(); // is 0 (no bit is set)

    return f;
}
