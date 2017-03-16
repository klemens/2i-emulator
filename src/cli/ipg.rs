use std::fs::File;
use std::path::Path;

use chrono::prelude::Local;
use clap::ArgMatches;
use emulator::parse::read_reachable_program;

pub fn main(args: &ArgMatches) -> Result<(), i32> {
    // Load the program from the given path
    let program_path = Path::new(args.value_of("2i-programm").unwrap());
    let program_file = File::open(program_path).map_err(|e| {
        println!("Die angegebene Datei konnte nicht geöffnet werden: {}", e);
        2
    })?;
    let program = read_reachable_program(&program_file).map_err(|e| {
        println!("Das Mikroprogramm konnte nicht geladen werden: {}", e);
        3
    })?;

    // Print header
    println!("#iP;31;{design_type},{min_version},{max_version};0;\
              Adr.;2;N5;1;1;3;4;1;1;1;1;4;1;Comment",
        design_type = "0201",
        min_version = "1360",
        max_version = "1369",
    );

    // Print signature on second line (ignored by mcontrol)
    println!("Erzeugt von 2i-emulator für MR 1.3.6;;;;;;;;;;;;;;;;;");

    // Print file path as comment and current date on third line
    println!("Mikroprogramm: {path};;;;;;;;;;;;;;;;;{date}",
        path = program_path.display(),
        date = Local::now().format("%d.%m.%Y %H:%M:%S"),
    );

    // Print all instructions
    for (i, &(addr, inst)) in program.iter().enumerate() {
        print!("{index};{address};\"{mnemonic}\";;{address:05b};",
            index = i,
            address = addr,
            mnemonic = inst.to_text_paraphrase(Some(addr as usize + 1)),
        );
        println!("{:02b};{:05b};{};{};{:03b};{:04b};{};{};{};{};{:04b};{};",
            inst.get_address_control(),
            inst.get_next_instruction_address(),
            inst.is_bus_writable() as u8,
            inst.is_bus_enabled() as u8,
            inst.get_register_address_a(),
            inst.get_constant_input() & 0b1111,
            inst.should_write_register_b() as u8,
            inst.should_write_register() as u8,
            inst.is_alu_input_a_bus() as u8,
            inst.is_alu_input_b_const() as u8,
            inst.get_alu_instruction(),
            inst.should_store_flags() as u8,
        );
    }

    Ok(())
}
