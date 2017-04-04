# 2i-emulator

Cli emulator for the micro computer 2i used in the computer science hardware
course at Leipzig University.

```
Register:        Eingaberegister:   Aktuelles Mikroprogramm:
  R0: 00000010     FC: 00000101       doc/examples/multiply.2i
  R1: 00001100     FD: 00001100
  R2: 00100100     FE: 00000000     Nächster Befehl (00110):
  R3: 00000000     FF: 00000000       00 01000 | 00 | 000 1111 01 | 01 0100 | 0
  R4: 00000000                        ~ R0 = R0 + FF; JMP 01000
  R5: 00000000   Ausgaberegister:
  R6: 00000000     FE: 00111100     Flag (Register) | Interrupt: A/010, B/111
  R7: 00000000     FF: 00000000       C: 0 (0), N: 0 (0), Z: 0 (0) | INT: 0, 0
```

## Build

The project can be built using `cargo`. You can also use one of the [binaries]
provided for convenience.

```sh
cargo build --release
```

## Usage

You can specify the filename of the program you want to load as a parameter on
the command line:

```sh
./2i-emulator doc/examples/answer.2i
```

Or use the built-in `load` command:

```sh
./2i-emulator
> load doc/examples/answer.2i
```

You can also generate LaTeX documents for your `2i`-programs:

```sh
./2i-emulator latex --autor "Erika Mustermann" answer.2i multiply.2i
```

See `./2i-emulator --help` for more details.

## Example

The following example (`answer.2i`) calculates the number `42` and writes it
to the output register `FE`.

```
# The answer to everything: (FE) = 42

00000: 00 00001 | 00 | 000 0101 01 | 01 0001 0
00001: 11 00010 | 00 | 000 0000 01 | 00 0100 0
00010: 00 00001 | 00 | 000 0000 01 | 00 0101 0
00011: 00 00100 | 00 | 000 0000 01 | 00 1000 0
00100: 00 00101 | 00 | 000 0000 01 | 00 1000 0
00101: 00 00110 | 00 | 001 1110 01 | 01 0001 0
00110: 00 00000 | 11 | 001 0000 00 | 00 0001 0
```

Any character but `0`, `1` and `:` is ignored inside commands and can be used
for formatting. Every command can optionally be prefixed with its address
followed by `:`. These must be strictly nondecreasing. Commands without an
explicit address are assinged the next one. An empty line and everything after
`#` is ignored.

## Documentation

The doc folder contains a german documentation of the micro computer and its
instructions. The [api documentation] of the emulator can be generated using
`cargo doc`.

## Licence

This program by Klemens Schölhorn is licenced under the terms of the GPLv3.

[binaries]: https://github.com/klemens/2i-emulator/releases
[api documentation]: https://klemens.github.io/2i-emulator/emulator/
