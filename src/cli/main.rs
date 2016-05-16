extern crate emulator;
extern crate regex;

use std::fs::File;
use std::io::{self, BufRead, Write};

use emulator::*;
use regex::Regex;

fn main() {
    let file_name = std::env::args().skip(1).next().unwrap();

    let program = parse::read_program(File::open(file_name).unwrap()).unwrap();

    // eg: FD = 1101
    let input_pattern = Regex::new(r"^(?P<index>F[C-F])\s+=\s+(?P<value>[01]{1,8})$").unwrap();

    let mut next_address = 0;
    let mut cpu = Cpu::new();
    let io = IoRegisters::new();
    let mut ram = Ram::new();
    ram.add_overlay(0xFC, 0xFF, &io);

    println!("2i-emulator v{}", option_env!("CARGO_PKG_VERSION").unwrap_or("*"));
    display_ui(&mut cpu, &io, next_address, program[next_address], None);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();

        if line.is_empty() {
            // execute next instruction
        } else if let Some(matches) = input_pattern.captures(line.trim()) {
            if let Ok(value) = u8::from_str_radix(&matches["value"], 2) {
                let mut input = io.inspect_input().borrow_mut();
                match &matches["index"] {
                    "FC" => input[0] = value,
                    "FD" => input[1] = value,
                    "FE" => input[2] = value,
                    "FF" => input[3] = value,
                    _ => panic!("Invalid regex match"),
                }
                print!("> ");
                io::stdout().flush().unwrap();
                continue;
            } else {
                print!("Ungültiger Wert.\n> ");
                io::stdout().flush().unwrap();
                continue;
            }
        } else {
            print!("Ungültige Eingabe.\n> ");
            io::stdout().flush().unwrap();
            continue;
        }

        // Execute next instruction and display the updated ui
        match cpu.execute_instruction(program[next_address], &mut ram) {
            Ok((na, flags)) => {
                next_address = na;
                display_ui(&mut cpu, &io, na, program[na], Some(flags));
            }
            Err(err) => {
                println!("Fehler beim Ausführen des Befehls: \"{}\"", err);
                return;
            }
        }
    }
}

/// Display the status UI of the cli
fn display_ui(cpu: &mut Cpu, io: &IoRegisters, next_instruction_address: usize,
              next_instruction: Instruction, flags: Option<Flags>) {
    let flag_register = cpu.inspect_flags().clone();
    let reg = cpu.inspect_registers();
    let input = io.inspect_input().borrow();
    let output = io.inspect_output().borrow();

    print!("
Register:        Eingaberegister:   Letzte Flags, Flag-Register:
  R0: {0:08b }     FC: {8:08b }       Carry: {co}, {cf} | Negativ: {no}, {nf} | Null: {zo}, {zf}
  R1: {1:08b }     FD: {9:08b }
  R2: {2:08b }     FE: {10:08b}     Nächster Befehl ({na:05b}):
  R3: {3:08b }     FF: {11:08b}       {instruction}
  R4: {4:08b }                        ~ {mnemonic}
  R5: {5:08b }   Ausgaberegister:
  R6: {6:08b }     FE: {12:08b}     [FC = 11010]: Eingaberegister ändern
  R7: {7:08b }     FF: {13:08b}     [ENTER]: Befehl ausführen

> ",
        reg[0], reg[1], reg[2], reg[3],
        reg[4], reg[5], reg[6], reg[7],
        input[0], input[1], input[2], input[3],
        output[0], output[1],
        instruction = format_instruction(next_instruction),
        mnemonic = next_instruction.to_text_paraphrase(Some(next_instruction_address + 1)),
        na = next_instruction_address,
        co = flags.map_or("-".into(), |f| format!("{}", f.carry() as u8)),
        no = flags.map_or("-".into(), |f| format!("{}", f.negative() as u8)),
        zo = flags.map_or("-".into(), |f| format!("{}", f.zero() as u8)),
        cf = flag_register.carry() as u8,
        nf = flag_register.negative() as u8,
        zf = flag_register.zero() as u8);
    io::stdout().flush().unwrap();
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
