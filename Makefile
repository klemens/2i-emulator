# compiler/linker settings
CC               = g++
override GFLAGS  +=
override CFLAGS  += -c -Wall -std=c++0x -O2 -Iinclude $(GFLAGS)
override LFLAGS  += -static $(GFLAGS)

# directories
SOURCE  = src
TESTS   = src/tests
HEADERS = include
OBJECTS = obj
EXEC    = .
UNIT    = .

default: $(EXEC)/2iEmulator.exe

unit: $(UNIT)/AluTest.exe

all: default unit


$(EXEC)/2iEmulator.exe: $(OBJECTS)/2iEmulator.o $(OBJECTS)/SoC.o $(OBJECTS)/Alu.o
	$(CC) -o $@ $(LFLAGS) $(OBJECTS)/2iEmulator.o $(OBJECTS)/SoC.o $(OBJECTS)/Alu.o

$(UNIT)/AluTest.exe: $(OBJECTS)/AluTest.o $(OBJECTS)/Alu.o
	$(CC) -o $@ $(LFLAGS) $(OBJECTS)/AluTest.o $(OBJECTS)/Alu.o


$(OBJECTS)/2iEmulator.o: $(SOURCE)/2iEmulator.cpp $(HEADERS)/Alu.h $(HEADERS)/SoC.h
	$(CC) -o $@ $(CFLAGS) $(SOURCE)/2iEmulator.cpp

$(OBJECTS)/SoC.o: $(SOURCE)/SoC.cpp $(HEADERS)/SoC.h
	$(CC) -o $@ $(CFLAGS) $(SOURCE)/SoC.cpp

$(OBJECTS)/Alu.o: $(SOURCE)/Alu.cpp $(HEADERS)/Alu.h
	$(CC) -o $@ $(CFLAGS) $(SOURCE)/Alu.cpp

$(OBJECTS)/AluTest.o: $(TESTS)/AluTest.cpp $(HEADERS)/Alu.h $(HEADERS)/MiniUnit.hpp
	$(CC) -o $@ $(CFLAGS) $(TESTS)/AluTest.cpp

$(HEADERS)/SoC.h: $(HEADERS)/Alu.h

clean:
	rm -f obj/*.o $(EXEC)/2iEmulator.exe $(UNIT)/AluTest.exe
