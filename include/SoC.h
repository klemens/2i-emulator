#ifndef _MINIRECHNER2I_SOC_H_
#define _MINIRECHNER2I_SOC_H_

#include <cstddef>
#include <bitset>

namespace Minirechner2i {

using std::size_t;
using std::bitset;

class SoC {
    public:
        void runInstruction();
        bitset<5> getNextInstructionNumber() { return nextInstruction; }

        bitset<25> getInstruction(size_t position);
        void setInstruction(size_t position, bitset<25> value);
        bitset<8> getRam(size_t position);
        void setRam(size_t position, bitset<8> value);
        bitset<8> getRegister(size_t position);

        bitset<8> getInputRegister(size_t position);
        void setInputRegister(size_t position, bitset<8> value);
        bitset<8> getOutputRegister(size_t position);
        void setOutputRegister(size_t position, bitset<8> value);


        size_t getInstructionRamSize() { return instructionRamSize; }
        size_t getRamSize() { return ramSize; }
        size_t getRegisterCount() { return registerCount; }
        size_t getInputRegisterCount() { return inputRegisterCount; }
        size_t getOutputRegisterCount() { return outputRegisterCount; }

    private:
        const static size_t ramSize = 252; // 2^8 - 4 Input register
        const static size_t instructionRamSize = 32; // 2^5 possible instructions
        const static size_t registerCount = 8; // 8 internal registers
        const static size_t inputRegisterCount = 4; // 4 input registers FC - FF
        const static size_t outputRegisterCount = 2; // 2 output registers FE - FF

        bitset<8> ram[ramSize];
        bitset<25> instructionRam[instructionRamSize];
        bitset<8> registers[registerCount];

        bitset<8> inputRegister[inputRegisterCount];
        bitset<8> outputRegister[outputRegisterCount];

        bitset<5> nextInstruction;
};

} // namespace Minirechner2i

#endif // _MINIRECHNER2I_SOC_H_
