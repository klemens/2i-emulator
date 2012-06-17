#include "Alu.h"
#include "MiniUnit.hpp"

#include <iostream>

namespace Minirechner2i {
namespace Tests {

using std::bitset;

class Alu : public MiniUnit::TestCase <Alu> {
    public:
        void testLogic() {
            bitset<4> leta("0000"), letb("0001"), nor("0010"), let0("0011");
            bitset<8> a("11010100"), b("00101101"), c;
            bitset<3> flags;

            // Cin = 0, a = 11010100, b = 00101101  -> f = 11010100, Ca = 0
            MiniUnitAssertEqual(alu->calculate(leta, a, b, flags), a);
            // Ca must be 0
            MiniUnitAssertEqual(flags[0], 0);
            // Cin = 0, a = 11010100, b = 00101101  -> f = 00101101, Ca = 0
            MiniUnitAssertEqual(alu->calculate(letb, a, b, flags), b);
            // Ca must be 0
            MiniUnitAssertEqual(flags[0], 0);
            // Cin = 0, a = 11010100, b = 00101101  -> f = 0, Ca = 0
            MiniUnitAssertEqual(alu->calculate(let0, a, b, flags), c);
            // Ca must be 0
            MiniUnitAssertEqual(flags[0], 0);

            // Cin = 0, a = 11010100, b = 00101101  -> f = 00000010, Ca = 0
            MiniUnitAssertEqual(alu->calculate(nor, a, b, flags), bitset<8>("00000010"));
            // Ca must be 0
            MiniUnitAssertEqual(flags[0], 0);
            // Cin = 0, a = b = 11010100  -> ~a = 00101011, Ca = 0
            MiniUnitAssertEqual(alu->calculate(nor, a, a, flags), bitset<8>("00101011"));
            // Ca must be 0
            MiniUnitAssertEqual(flags[0], 0);
        }

        void testSummation() {
            bitset<4> add("0100"), addo("0101"), addc("0110"), addci("0111");
            unsigned long a = 47, b = 19, c = 236;
            bitset<3> flags;

            // Cin = 0, a = 47, b = 19  -> a + b = 66, Ca = 0
            MiniUnitAssertEqual(alu->calculate(add, a, b, flags), bitset<8>(66));
            // Ca must be 0
            MiniUnitAssertEqual(flags[0], 0);
            // Cin = 0, a = 47, b = 19  -> a + b + 1 = 67, Ca = 1
            MiniUnitAssertEqual(alu->calculate(addo, a, b, flags), bitset<8>(67));
            // Ca must be 1
            MiniUnitAssertEqual(flags[0], 1);
            // Cin = 1, a = 47, b = 19  -> a + b + Cin = 67, Ca = 0
            MiniUnitAssertEqual(alu->calculate(addc, a, b, flags), bitset<8>(67));
            // Ca must be 0
            MiniUnitAssertEqual(flags[0], 0);
            // Cin = 0, a = 47, b = 19  -> a + b + ~Cin = 67, Ca = 1
            MiniUnitAssertEqual(alu->calculate(addci, a, b, flags), bitset<8>(67));
            // Ca must be 1
            MiniUnitAssertEqual(flags[0], 1);
            // Cin = 1, a = 47, c = 236  -> a + c + Cin = 28 (overflow), Ca = 1
            MiniUnitAssertEqual(alu->calculate(addc, a, c, flags), bitset<8>(28));
            // Ca must be 1
            MiniUnitAssertEqual(flags[0], 1);
        }

        void testShift() {
            bitset<4> lsr("1000"), rr("1001"), rrc("1010"), asr("1011");
            bitset<8> a("11010110"), b("00101101"), c;
            bitset<3> flags;

            // Cin = 0, a = 11010110  -> a >> 1 = 01101011, Ca = 0
            MiniUnitAssertEqual(alu->calculate(lsr, a, c, flags), bitset<8>("01101011"));
            // Ca must be 0
            MiniUnitAssertEqual(flags[0], 0);
            // Cin = 0, b = 00101101  -> b RR 1 = 10010110, Ca = 1
            MiniUnitAssertEqual(alu->calculate(rr, b, c, flags), bitset<8>("10010110"));
            // Ca must be 1
            MiniUnitAssertEqual(flags[0], 1);
            // Cin = 1, a = 11010110  -> a RRC 1 = 11101011, Ca = 0
            MiniUnitAssertEqual(alu->calculate(rrc, a, c, flags), bitset<8>("11101011"));
            // Ca must be 0
            MiniUnitAssertEqual(flags[0], 0);
            // Cin = 0, b = 00101101  -> b >>A 1 = 00010110, Ca = 1
            MiniUnitAssertEqual(alu->calculate(asr, b, c, flags), bitset<8>("00010110"));
            // Ca must be 1
            MiniUnitAssertEqual(flags[0], 1);
        }

        void testFlags() {
            bitset<4> leta("0000"), clc("1100"), setc("1101"), letc("1110"), invc("1111");
            bitset<8> a("11010110"), b("00101101"), c;
            bitset<3> flags;

            // Cin = 0  -> set carry, Ca = 1
            alu->calculate(setc, c, c, flags);
            // Ca must be 1
            MiniUnitAssertEqual(flags[0], 1);
            // Cin = 1  -> let carry, Ca = 1
            alu->calculate(letc, c, c, flags);
            // Ca must be 1
            MiniUnitAssertEqual(flags[0], 1);
            // Cin = 1  -> clear carry, Ca = 0
            alu->calculate(clc, c, c, flags);
            // Ca must be 0
            MiniUnitAssertEqual(flags[0], 0);
            // Cin = 0  -> invert carry, Ca = 1
            alu->calculate(invc, c, c, flags);
            // Ca must be 1
            MiniUnitAssertEqual(flags[0], 1);

            // c = 0 -> N = 0, Z = 1
            alu->calculate(leta, c, c, flags);
            MiniUnitAssertEqual(flags[1], 0);
            MiniUnitAssertEqual(flags[2], 1);
            // b = 00101101 (45) -> N = 0, Z = 0
            alu->calculate(leta, b, c, flags);
            MiniUnitAssertEqual(flags[1], 0);
            MiniUnitAssertEqual(flags[2], 0);
            // a = 11010110 (-42) -> N = 1, Z = 0
            alu->calculate(leta, a, c, flags);
            MiniUnitAssertEqual(flags[1], 1);
            MiniUnitAssertEqual(flags[2], 0);
        }

        void initialize() {
            alu = new Minirechner2i::Alu;
        }

        void clean() {
            delete alu;
            alu = NULL;
        }

    private:
        Minirechner2i::Alu * alu;
};

}
}

int main(int argc, char** args) {
    typedef Minirechner2i::Tests::Alu AluTest;

    AluTest test;

    test.addTest(&AluTest::testLogic, "nor and let");
    test.addTest(&AluTest::testSummation, "summation");
    test.addTest(&AluTest::testShift, "right shift");
    test.addTest(&AluTest::testFlags, "flags");

    std::cout << test.run();

    return 0;
}
