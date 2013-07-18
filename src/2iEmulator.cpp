#include "Alu.h"
#include "SoC.h"

#include <iostream>
#include <vector>
#include <sstream>
#include <fstream>

namespace Minirechner2i {

using std::endl;
using std::string;
using std::vector;
using std::ostringstream;
using std::istringstream;

class ConsoleRunner {
    public:
        ConsoleRunner(SoC soc) : soc(soc) {}

        int run() {
            std::istream & in = std::cin;
            std::ostream & out = std::cout;

            out << "2iEmulator - Emulator fuer den Minirechner2i" << endl << endl;

            readInstructions(in, out);

            while(true) {
                out << endl;
                displayOverview(out);
                out << endl << "> ";

                string cmd;
                getline(in, cmd);

                if(cmd == "i") // Change input register
                    setInputRegister(in, out);
                else if(cmd == "r") // RamInspector
                    out << "Not implemented yet" << endl; //todo
                else if(cmd == "q") // quit
                    break;
                else
                    soc.runInstruction();
            }

            return 0;
        }

    private:
        void readInstructions(std::istream & in, std::ostream & out) {
            out << "Bitte der Reihe nach die Befehle oder einen Dateinamen eingeben:" << endl
                << "(\"q\" zum Ueberspringen der restlichen, fuehrende Nullen weglassen," << endl
                << "es kann auch eine Datei angegeben werden, siehe Beispiele)"
                << endl << endl;

            for(size_t i = 0; i < 32; ++i) {
                out << bitset<5>(i) << " (" << i << "): ";

                string line;
                getline(in, line);

                // Check it the user entered a filename
                std::ifstream ifs(line, std::ifstream::in);
                if(ifs.good()) {
                    out << "Lese aus Datei \"" << line << "\":" << endl;
                    ostringstream o;
                    readInstructions(ifs, o);
                    return;
                }

                if(line == "q") break;
                else {
                    try {
                        bitset<25> instruction(line);
                        soc.setInstruction(i, instruction);
                        out << "\r\r" << instruction << endl;
                    } catch(...) {
                        --i;
                        continue;
                    }
                }
            }
        }

        void setInputRegister(std::istream & in, std::ostream & out) {
            out << "Eingaberegister waehlen (0-3): ";

            size_t i;
            do {
                string line;
                getline(in, line);
                istringstream iStream(line);
                iStream >> i;
            } while(i >= 4);

            out << std::hex << std::uppercase << (i + 252) << std::dec << " = ";

            bitset<8> value;
            string line;
            getline(in, line);
            istringstream iStream(line);
            iStream >> value;

            soc.setInputRegister(i, value);
        }

        void displayOverview(std::ostream & out) {
            // Insert registers
            vector<string> registers;
            registers.push_back("Register:        ");
            for(int i = 0; i < 8; ++i) {
                ostringstream out;
                out << bitset<3>(i).to_string() << " (" << i << "): "
                    << soc.getRegister(i);
                registers.push_back(out.str());
            }

            // In/Output registers
            vector<string> ioRegisters;
            ioRegisters.push_back("Eingaberegister:");
            for(int i = 0; i < 4; ++i) {
                ostringstream out;
                out << std::uppercase << std::hex << (i + 252) << std::dec
                    << " (" << i << "): " << soc.getInputRegister(i);
                ioRegisters.push_back(out.str());
            }
            ioRegisters.push_back("                ");
            ioRegisters.push_back("Ausgaberegister:");
            for(int i = 0; i < 2; ++i) {
                ostringstream out;
                out << std::uppercase << std::hex << (i + 254) << std::dec
                    << " (" << i << "): " << soc.getOutputRegister(i);
                ioRegisters.push_back(out.str());
            }

            // Flags, next instruction, help
            vector<string> rest;
            rest.push_back("Flagregister:");
            {
                ostringstream out;
                out << "Carry: " << soc.getFlag(SoC::CARRY_FLAG) << " | Negativ: "
                    << soc.getFlag(SoC::NEGATIVE_FLAG) << " | Null (Z): "
                    << soc.getFlag(SoC::ZERO_FLAG);
                rest.push_back(out.str());
            }
            rest.push_back("");
            {
                ostringstream out;
                out << "Naechster Befehl: " << soc.getNextInstructionNumber() << " ("
                    << soc.getNextInstructionNumber().to_ulong() << "):";
                rest.push_back(out.str());
            }
            {
                ostringstream out;
                bitset<25> i = soc.getInstruction(soc.getNextInstructionNumber().to_ulong());
                out << "  " << SoC::substr<2>(i, 23) << " " << SoC::substr<5>(i, 18)
                    << "|" << i[17] << i[16] << "|" << SoC::substr<3>(i, 13) << " "
                    << SoC::substr<4>(i, 9) << " " << i[8] << i[7] << "|" << i[6]
                    << " " << i[5] << " " << SoC::substr<4>(i, 1) << "|" << i[0];
                rest.push_back(out.str());
            }
            rest.push_back("");
            rest.push_back("[i]: Eingaberegister aendern");
            rest.push_back("[r]: RamInspector   [q]: Beenden");
            rest.push_back("[ENTER]: Befehl ausfuehren");

            vector<string> spacer; spacer.insert(spacer.begin(), 9, "     ");
            vector<string> full = appendByLine(appendByLine(appendByLine(
                                  appendByLine(registers, spacer), ioRegisters), spacer), rest);

            for(auto it = full.begin(); it != full.end(); ++it)
                out << *it << endl;
        }

        vector<string> appendByLine(vector<string> a, vector<string> b) {
            vector<string> ret;

            auto itA = a.begin();
            auto itB = b.begin();
            for(; itA != a.end() && itB != b.end(); ++itA, ++itB) {
                ret.push_back(*itA + *itB);
            }

            return ret;
        }

        SoC & soc;
};

} // namespace Minirechner2i


using Minirechner2i::Alu;
using Minirechner2i::SoC;
using Minirechner2i::ConsoleRunner;

int main() {
    Alu alu;
    SoC soc(alu);

    ConsoleRunner prog(soc);

    return prog.run();
}
