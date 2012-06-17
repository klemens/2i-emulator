#include "Alu.h"
#include "MiniUnit.hpp"

#include <iostream>

namespace Minirechner2i {
namespace Tests {

using std::bitset;

class Alu : public MiniUnit::TestCase <Alu> {
    public:
        void testSummation() {
            unsigned long a = 47, b = 19, c = 236;
            bitset<3> flags;
            bitset<4> add("0100"), addo("0101"), addc("0110"), addci("0111");

            // Cin = 0, a = 47, b = 19  -> a + b = 66, Ca = 0
            MiniUnitAssertEqual(alu->calculate(add, a, b, flags), bitset<8>(66));
            // Cin = 0, a = 47, b = 19  -> a + b + 1 = 67, Ca = 1
            MiniUnitAssertEqual(alu->calculate(addo, a, b, flags), bitset<8>(67));
            // Cin = 1, a = 47, b = 19  -> a + b + Cin = 67, Ca = 0
            MiniUnitAssertEqual(alu->calculate(addc, a, b, flags), bitset<8>(67));
            // Cin = 0, a = 47, b = 19  -> a + b + ~Cin = 67, Ca = 1
            MiniUnitAssertEqual(alu->calculate(addci, a, b, flags), bitset<8>(67));

            // Cin = 1, a = 47, c = 236  -> a + c + Cin = 28 (overflow), Ca = 1
            MiniUnitAssertEqual(alu->calculate(addc, a, c, flags), bitset<8>(28));
            // Ca must be 1
            MiniUnitAssertEqual(flags[0], 1);
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

    test.addTest(&AluTest::testSummation, "Sum");
    std::cout << test.run();

    return 0;
}
