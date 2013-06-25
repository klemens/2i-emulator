# 2iEmulator #

This is an emulator for the micro copmuter used in the hardware practicum of the
computer science study in University of Leipzig.

## Build ##

The provided Makefile works with g++ under both linux and windows.
To build the unit tests, build the target __unit__.

## Usage ##

The program provides a mostly self-explaining console interface.

When the program asks for the first instruction, you can also enter a filename
which contains one instustion per line in ascii. (see examples)

## Documentation ##

In doc you'll find a german document describing the micro computer and its
instructions. The header files also include some basic information.

## Examples ##

doc/examples contains two example programs: 

1. __add2__:Reads two numbers from FC and FD and writes (FC + FD) * 2 into FF.
2. __mul.txt__: Reads two numbers from FC and FD and writes their product into FE.

## Licence ##

This program by Klemens Schölhorn is licenced under GPLv3.
