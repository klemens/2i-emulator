#include "Alu.h"
#include "SoC.h"

namespace Minirechner2i {

class ConsoleRunner {
    public:
        ConsoleRunner(SoC soc) : soc(soc) {}

        int run() {
            return 0;
        }

    private:
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
