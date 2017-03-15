use std::borrow::Cow;
use std::io::{self, Write};
use std::path::Path;

use emulator::{Flags, Instruction, IoRegisters, Ram};
use super::*;

/// Display the status UI of the cli
pub fn status(computer: &mut Computer, io: &IoRegisters, program: &Program,
              flags: Option<Flags>) {
    let flag_register = computer.cpu.inspect_flags().clone();
    let reg = computer.cpu.inspect_registers();
    let input = io.inspect_input().borrow();
    let output = io.inspect_output().borrow();
    let next_instruction = program.instructions[computer.instruction_pointer];

    let path = ellipsize_path(&program.path, 41);

    print!("
Register:        Eingaberegister:   Aktuelles Mikroprogramm:
  R0: {0:08b }     FC: {8:08b }       {program_path}
  R1: {1:08b }     FD: {9:08b }
  R2: {2:08b }     FE: {10:08b}     Nächster Befehl ({ip:05b}):
  R3: {3:08b }     FF: {11:08b}       {instruction}
  R4: {4:08b }                        ~ {mnemonic}
  R5: {5:08b }   Ausgaberegister:
  R6: {6:08b }     FE: {12:08b}     Letzte Flags, Flag-Register:
  R7: {7:08b }     FF: {13:08b}       Carry: {co}, {cf} | Negativ: {no}, {nf} | Null: {zo}, {zf}

",
        reg[0], reg[1], reg[2], reg[3],
        reg[4], reg[5], reg[6], reg[7],
        input[0], input[1], input[2], input[3],
        output[0], output[1],
        program_path = path,
        instruction = format_instruction(next_instruction),
        mnemonic = next_instruction.to_text_paraphrase(Some(computer.instruction_pointer + 1)),
        ip = computer.instruction_pointer,
        co = flags.map_or("-".into(), |f| format!("{}", f.carry() as u8)),
        no = flags.map_or("-".into(), |f| format!("{}", f.negative() as u8)),
        zo = flags.map_or("-".into(), |f| format!("{}", f.zero() as u8)),
        cf = flag_register.carry() as u8,
        nf = flag_register.negative() as u8,
        zf = flag_register.zero() as u8);
    io::stdout().flush().unwrap();
}

/// Ellipsize path if necessary
fn ellipsize_path(path: &Path, max_length: usize) -> Cow<str> {
    assert!(max_length >= 4);

    let path_string = path.to_string_lossy();
    // This only works for ascii-like strings
    if path_string.chars().count() > max_length {
        let rev_short: String = path_string.chars().rev().take(max_length - 3).collect();
        "...".chars().chain(rev_short.chars().rev()).collect()
    } else {
        path_string
    }
}

/// Format the given instruction as a logically grouped "binary" string
fn format_instruction(inst: Instruction) -> String {
    format!("{:02b} {:05b} | {}{} | {:03b} {:04b} {}{} | {}{} {:04b} | {}",
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
        inst.should_store_flags() as u8)
}

/// Print a overview of the ram in common hex-editor format
pub fn display_ram(ram: &Ram) {
    println!("\n    _0 _1 _2 _3 _4 _5 _6 _7 _8 _9 _A _B _C _D _E _F");

    for (i, cell) in ram.inspect().borrow()[0..252].iter().enumerate() {
        if i % 16 == 0 {
            print!("\n{:X}_ ", i / 16);
        }
        print!(" {:02X}", cell);
    }

    println!("\n");
}

/// Display a list of all commands with descriptions
pub fn display_help() {
    println!("\n\
        FX = <value>  Eingaberegister setzen (zB: FC = 11010)\n\
        ENTER         Nächsten Befehl ausführen\n\
        ram           RAM-Übersicht anzeigen\n\
        help          Hilfe anzeigen\n\
        exit/quit     Emulator beenden (alternativ: STRG-D)\n")
}