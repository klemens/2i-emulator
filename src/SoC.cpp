#include "SoC.h"

#include <stdexcept>

using std::bitset;
using std::logic_error;

void Minirechner2i::SoC::runInstruction() {
    // Get current instruction
    bitset<25> cur = instructionRam[nextInstruction.to_ulong()];
    bitset<8> a, b, f;

    // Determine input a of ALU
    bitset<8> aRegisterValue = registers[SoC::substr<3>(cur, 13).to_ulong()];
    if(cur[6]) { // Read from RAM (with address from register)
        if(!cur[16])
            throw logic_error("Cannot read from disabled bus!");
        else if(cur[17])
            throw logic_error("Cannot read from write-only bus!");

        if(aRegisterValue.to_ulong() >= 0xFC) // Input register
            a = inputRegister[aRegisterValue.to_ulong() - 0xFC];
        else // RAM
            a = ram[aRegisterValue.to_ulong()];
    } else { // Read from register
        a = aRegisterValue;
    }

    // Determine input b of ALU
    if(cur[5]) { // Read as constant
        bitset<4> bTmp = SoC::substr<4>(cur, 9);
        if(bTmp[3]) { // If bit 3 is set, set bits 4 - 7 to 1 (1111 -> FF)
            b.set();
        }
        b[2] = bTmp[2]; b[1] = bTmp[1]; b[0] = bTmp[0];
    } else { // Read from register
        b = registers[SoC::substr<3>(cur, 9).to_ulong()];
    }

    // Calculate output of alu
    bitset<3> flagsNew = flags;
    f = alu.calculate(SoC::substr<4>(cur, 1), a, b, flagsNew);

    // Save back to registers
    if(cur[7]) {
        if(cur[8]) // Write to b
            registers[SoC::substr<3>(cur, 9).to_ulong()] = f;
        else // Write to a
            registers[SoC::substr<3>(cur, 13).to_ulong()] = f;
    }

    //Save back to RAM
    if(cur[16] && cur[17]) { // If bus is enabled and write-only
        size_t address = registers[SoC::substr<3>(cur, 13).to_ulong()].to_ulong();

        if(address == 0xFC || address == 0xFD)
            throw logic_error("Cannot write into input register!");

        if(address >= 0xFE) // Write into output register
            outputRegister[address - 0xFE] = f;
        else // Write into RAM
            ram[address] = f;
    }

    // Save output flags if desired
    if(cur[0])
        flags = flagsNew;

    // Calculate next address
    nextInstruction = calculateNextAddress(SoC::substr<5>(cur, 18), SoC::substr<2>(cur, 23), flagsNew, flags);
}

bitset<5> Minirechner2i::SoC::calculateNextAddress(bitset<5> next, bitset<2> mac,
                                                   bitset<3> falgs, bitset<3> flagRegister) {
    bitset<5> ret = next;

    // func = MAC<<1 | N0
    bitset<3> func; func[2] = mac[1]; func[1] = mac[0]; func[0] = next[0];

    switch(func.to_ulong()) {
        case 0: // 000
        case 1: // 001
            ret[0] = next[0];
            break;
        case 2: // 010
            ret[0] = 1;
            break;
        case 3: // 011
            ret[0] = flagRegister[0];
            break;
        case 4: // 100
            ret[0] = falgs[0];
            break;
        case 5: // 101
            ret[0] = falgs[2];
            break;
        case 6: // 110
            ret[0] = falgs[1];
            break;
        case 7: // 111
            ret[0] = 0;
            break;
    }

    return ret;
}

bitset<25> Minirechner2i::SoC::getInstruction(size_t position) {
    if(position < instructionRamSize)
        return instructionRam[position];
    else
        throw std::out_of_range("Minirechner2i::SoC::getInstruction");
}

void Minirechner2i::SoC::setInstruction(size_t position, bitset<25> value) {
    if(position < instructionRamSize)
        instructionRam[position] = value;
    else
        throw std::out_of_range("Minirechner2i::SoC::setInstruction");
}

bitset<8> Minirechner2i::SoC::getRam(size_t position) {
    if(position < ramSize)
        return ram[position];
    else
        throw std::out_of_range("Minirechner2i::SoC::getRam");
}

void Minirechner2i::SoC::setRam(size_t position, bitset<8> value) {
    if(position < ramSize)
        ram[position] = value;
    else
        throw std::out_of_range("Minirechner2i::SoC::setRam");

}

bitset<8> Minirechner2i::SoC::getRegister(size_t position) {
    if(position < registerCount)
        return registers[position];
    else
        throw std::out_of_range("Minirechner2i::SoC::getRegister");
}

bitset<8> Minirechner2i::SoC::getInputRegister(size_t position) {
    if(position < inputRegisterCount)
        return inputRegister[position];
    else
        throw std::out_of_range("Minirechner2i::SoC::getInputRegister");
}

void Minirechner2i::SoC::setInputRegister(size_t position, bitset<8> value) {
    if(position < inputRegisterCount)
        inputRegister[position] = value;
    else
        throw std::out_of_range("Minirechner2i::SoC::setInputRegister");
}

bitset<8> Minirechner2i::SoC::getOutputRegister(size_t position) {
    if(position < outputRegisterCount)
        return outputRegister[position];
    else
        throw std::out_of_range("Minirechner2i::SoC::getOutputRegister");
}
