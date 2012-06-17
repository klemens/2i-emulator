#include "SoC.h"

#include <stdexcept>

using std::bitset;

void Minirechner2i::SoC::runInstruction() {
    //determine input A of ALU
    //determine input B of ALU

    //calculate output of alu

    //save back to registers and/or ram

    //save output flags if requested

    //calculate next address
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

bool Minirechner2i::SoC::setInputRegister(size_t position, bitset<8> value) {
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

bool Minirechner2i::SoC::setOutputRegister(size_t position, bitset<8> value) {
    if(position < outputRegisterCount)
        outputRegister[position] = value;
    else
        throw std::out_of_range("Minirechner2i::SoC::setOutputRegister");
}
