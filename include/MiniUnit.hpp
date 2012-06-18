#ifndef _MINIRECHNER2I_TESTS_UNIT_
#define _MINIRECHNER2I_TESTS_UNIT_

#include <vector>
#include <exception>
#include <sstream>

namespace MiniUnit {

class Exception : public std::exception {
    public:
        Exception(const std::string & message) {
            this->message = message;
        }
        virtual ~Exception() throw() {}

        virtual const char * what() const throw() {
            return message.c_str();
        }

    private:
        std::string message;
};

template <typename T>
class TestCase {
    public:
        typedef void (T::*testfunction_t) ();

        std::string run() {
            std::ostringstream r;
            unsigned int long passed = 0;

            T * testSuite = dynamic_cast<T*>(this);

            r << "Running " << testFunctions.size() << " unit tests.\n";

            auto itF = testFunctions.begin();
            auto itN = testNames.begin();
            for(;itF != testFunctions.end(); ++itF, ++itN) {
                initialize();

                try {
                    (testSuite->*(*itF))();
                    ++passed;
                } catch(Exception& e) {
                    r << "Failed test \"" << *itN << "\": " << e.what() << "\n";
                } catch(std::exception& e) {
                    r << "Error during test \"" << *itN << "\": " << e.what() << "\n";
                } catch(...) {
                    r << "Unknown error during test \"" << *itN << "\"\n";
                }

                clean();
            }

            if(passed == testFunctions.size())
                r << "Passed all unit tests!\n";
            else
                r << "Passed " << passed << " of " << testFunctions.size()
                  << " unit tests.\n";

            return r.str();
        }

        void addTest(testfunction_t function, std::string name) {
            testFunctions.push_back(function);
            testNames.push_back(name);
        }

    protected:
        void virtual initialize() = 0;
        void virtual clean() = 0;

    private:
        std::vector<testfunction_t> testFunctions;
        std::vector<std::string> testNames;
};

void assertFail(const char * desc, const char * file, int line) {
    std::ostringstream r;

    r << desc << " (" << file << ":" << line << ")";

    throw Exception(r.str());
}

template <typename L>
void assertTrue(const L & expr, const char * code, const char * file, int line) {
    if(!expr) {
        std::ostringstream r;

        r << "Assert \"" << code << "\" failed! (" << file << ":" << line << ")";

        throw Exception(r.str());
    }
}

template <typename L, typename R>
void assertEqual(const L & lExpr, const R & rExpr, bool comparison,
                 const char * lCcode, const char * rCcode, const char * file, int line) {
    if((lExpr == rExpr) != comparison) {
        std::ostringstream r;
        const char * comparisonOperator = (comparison ? " == " : " != ");

        r << "Assert \"" << lCcode << comparisonOperator << rCcode << "\" ("
          << lExpr << comparisonOperator << rExpr << ") failed! ("
          << file << ":" << line << ")";

        throw Exception(r.str());
    }
}

#define MiniUnitAssert(X) MiniUnit::assertTrue(X, #X, __FILE__, __LINE__)
#define MiniUnitFail(NAME) MiniUnit::assertFail(NAME, __FILE__, __LINE__)
#define MiniUnitAssertEqual(X, Y) MiniUnit::assertEqual(X, Y, true, #X, #Y, __FILE__, __LINE__)
#define MiniUnitAssertUnequal(X, Y) MiniUnit::assertEqual(X, Y, false, #X, #Y, __FILE__, __LINE__)

}

#endif // _MINIRECHNER2I_TESTS_UNIT_
