#include "Cpu.h"

#include <stdexcept>

void Minirechner2i::Cpu::runInstruction() {
    //determine input A of ALU
    //determine input B of ALU

    //calculate output of alu

    //save back to registers and/or ram

    //save output flags if requested

    //calculate next address
}

bitset<25> Minirechner2i::Cpu::getInstruction(size_t position) {
    if(position < instructionRamSize)
        return instructionRam[position];
    else
        throw std::out_of_range("Minirechner2i::Cpu::getInstruction");
}

void Minirechner2i::Cpu::setInstruction(size_t position, bitset<25> value) {
    if(position < instructionRamSize)
        instructionRam[position] = value;
    else
        throw std::out_of_range("Minirechner2i::Cpu::setInstruction");
}

bitset<8> Minirechner2i::Cpu::getRam(size_t position) {
    if(position < ramSize)
        return ram[position];
    else
        throw std::out_of_range("Minirechner2i::Cpu::getRam");
}

void Minirechner2i::Cpu::setRam(size_t position, bitset<8> value) {
    if(position < ramSize)
        ram[position] = value;
    else
        throw std::out_of_range("Minirechner2i::Cpu::setRam");

}

bitset<8> Minirechner2i::Cpu::getRegister(size_t position) {
    if(position < registerCount)
        return registers[position];
    else
        throw std::out_of_range("Minirechner2i::Cpu::getRegister");
}

bitset<8> Minirechner2i::Cpu::getInputRegister(size_t position) {
    if(position < inputRegisterCount)
        return inputRegister[position];
    else
        throw std::out_of_range("Minirechner2i::Cpu::getInputRegister");
}

bool Minirechner2i::Cpu::setInputRegister(size_t position, bitset<8> value) {
    if(position < inputRegisterCount)
        inputRegister[position] = value;
    else
        throw std::out_of_range("Minirechner2i::Cpu::setInputRegister");
}

bitset<8> Minirechner2i::Cpu::getOutputRegister(size_t position) {
    if(position < outputRegisterCount)
        return outputRegister[position];
    else
        throw std::out_of_range("Minirechner2i::Cpu::getOutputRegister");
}

bool Minirechner2i::Cpu::setOutputRegister(size_t position, bitset<8> value) {
    if(position < outputRegisterCount)
        outputRegister[position] = value;
    else
        throw std::out_of_range("Minirechner2i::Cpu::setOutputRegister");
}
