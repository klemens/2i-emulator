# compiler/linker settings
CC               = g++
override GFLAGS  +=
override CFLAGS  += -c -Wall -std=c++11 -O2 -Iinclude $(GFLAGS)
override LFLAGS  += -static $(GFLAGS)

# directories
SOURCE  = src
TESTS   = src/tests
HEADERS = include
OBJECTS = obj
EXEC    = .
UNIT    = .

all: unit

unit: $(UNIT)/AluTest.exe


$(UNIT)/AluTest.exe: $(OBJECTS)/AluTest.o $(OBJECTS)/Alu.o
	$(CC) -o $@ $(LFLAGS) $(OBJECTS)/AluTest.o $(OBJECTS)/Alu.o


$(OBJECTS)/Alu.o: $(SOURCE)/Alu.cpp $(HEADERS)/Alu.h
	$(CC) -o $@ $(CFLAGS) $(SOURCE)/Alu.cpp

$(OBJECTS)/AluTest.o: $(TESTS)/AluTest.cpp $(HEADERS)/Alu.h $(HEADERS)/MiniUnit.hpp
	$(CC) -o $@ $(CFLAGS) $(TESTS)/AluTest.cpp
