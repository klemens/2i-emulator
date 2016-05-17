# 2iEmulator

Emulator for the micro computer used in the computer science hardware course
at Leipzig University.

## Build

The project can be built using `cargo`. You can also use one of the [binaries]
provided for convenience.

```sh
cargo build --release
```

## Usage

You have to run the emulator in a terminal and specify the filename of the
program you want to run as a parameter:

```sh
target/release/2i-emulator-cli doc/examples/answer.2i
```

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

Any character but `0` and `1` is ignored inside commands and can be used for
formatting. Every command can optionally be prefixed with its address. Empty
lines and ones that start with `#` are ignored.

## Documentation

The doc folder contains a german documentation of the micro computer and its
instructions. The [api documentation] of the emulator can be generated using
`cargo doc`.

## Licence

This program by Klemens Schölhorn is licenced under the terms of the GPLv3.

[binaries]: https://github.com/klemens/2iEmulator/releases
[api documentation]: https://klemens.github.io/2iEmulator/emulator/
