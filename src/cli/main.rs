extern crate emulator;

use std::fs::File;

use emulator::parse;

fn main() {
    let file_name = std::env::args().skip(1).next().unwrap();

    let program = parse::read_program(File::open(file_name).unwrap()).unwrap();

    println!("{:#?}", program);
}
