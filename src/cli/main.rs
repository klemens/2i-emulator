extern crate emulator;
extern crate regex;
extern crate rustyline;

mod ui;

use std::fs::File;
use std::path::PathBuf;

use regex::Regex;

fn main() {
    // Load the program from the filename given as the first cli parameter
    let program = if let Some(file_name) = std::env::args().skip(1).next() {
        let path = PathBuf::from(file_name);
        if let Ok(file) = File::open(&path) {
            match emulator::parse::read_program(file) {
                Ok(program) => Program { path: path, instructions: program },
                Err(err) => {
                    println!("Fehler beim Laden des Programms: {}", err);
                    return;
                }
            }
        } else {
            println!("Die angegebene Datei konnte nicht geöffnet werden.");
            return;
        }
    } else {
        println!("Es wurde kein Programm zum Laden angegeben. Bitte geben Sie \
                  den Dateinamen eines Programmes als Kommandozeilenparameter \
                  an.\n\nEin Programm besteht aus einer Textdatei mit einem \
                  Befehl pro Zeile. Leere Zeilen und solche, die mit einem # \
                  beginnen, werden ignoriert. Alle Zeichen außer 0 und 1 \
                  werden innerhalb eines Befehls ignoriert und können zur \
                  Formatierung verwendet werden. Befehlen kann optional ihre \
                  Adresse vorangestellt werden. Beispiel:\n\n\
                  # R0 = (FC)\n\
                  \n       00,00001 00 000|1100 01 01,0001 0\n\
                  00001: 00,00000 01 000|0000 01 10,0000 0");
        return;
    };

    let io = emulator::IoRegisters::new();
    let mut computer = Computer::new(&io);

    println!("2i-emulator {}, GPLv3, https://github.com/klemens/2i-emulator",
             option_env!("CARGO_PKG_VERSION").unwrap_or("*"));
    ui::status(&mut computer, &io, &program, None);

    let config = rustyline::Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();
    let completer = Completer{};
    let mut line_reader = rustyline::Editor::with_config(config);
    line_reader.set_completer(Some(&completer));

    // eg: FD = 1101
    let input_pattern = Regex::new(r"^(?P<index>F[C-F])\s+=\s+(?P<value>[01]{1,8})$").unwrap();

    while let Ok(line) = line_reader.readline("> ") {
        let line = line.trim();

        // Add all non-empty inputs to the history
        if ! line.is_empty() {
            line_reader.add_history_entry(line.as_ref());
        }

        if line.is_empty() {
            // Execute next instruction and display the updated ui
            match computer.step(&program) {
                Ok(flags) => ui::status(&mut computer, &io, &program, Some(flags)),
                Err(err) => {
                    println!("Fehler beim Ausführen des Befehls: \"{}\"", err);
                    return;
                }
            }
        } else if line == "exit" || line == "quit" {
            return;
        } else if line == "help" {
            ui::display_help();
        } else if line == "ram" {
            ui::display_ram(&computer.ram);
        } else if let Some(matches) = input_pattern.captures(line) {
            // Try to set one of the input registers
            if let Ok(value) = u8::from_str_radix(&matches["value"], 2) {
                match &matches["index"] {
                    "FC" => io.inspect_input().borrow_mut()[0] = value,
                    "FD" => io.inspect_input().borrow_mut()[1] = value,
                    "FE" => io.inspect_input().borrow_mut()[2] = value,
                    "FF" => io.inspect_input().borrow_mut()[3] = value,
                    _ => panic!("Invalid regex match"),
                }
                ui::status(&mut computer, &io, &program, None);
            } else {
                println!("Ungültiger Wert.");
            }
        } else {
            println!("Ungültige Eingabe. \"help\" für Hilfe.");
        }
    }
}

#[derive(Default)]
pub struct Computer<'a> {
    cpu: emulator::Cpu,
    instruction_pointer: usize,
    ram: emulator::Ram<'a>,
}

impl<'a> Computer<'a> {
    fn new(io: &'a emulator::IoRegisters) -> Computer<'a> {
        let mut computer = Computer::default();
        computer.ram.add_overlay(0xFC, 0xFF, io);
        computer
    }

    /// Execute next instruction and update the instruction pointer
    fn step(&mut self, program: &Program) -> emulator::Result<emulator::Flags> {
        let instruction = program.instructions[self.instruction_pointer];
        self.cpu.execute_instruction(instruction, &mut self.ram).map(|(ip, flags)| {
            self.instruction_pointer = ip;
            flags
        })
    }
}

pub struct Program {
    path: PathBuf,
    instructions: [emulator::Instruction; 32],
}

struct Completer {}

impl rustyline::completion::Completer for Completer {
    fn complete(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<String>)> {
        // only complete at the end
        if pos < line.len() {
            return Ok((0, vec![]));
        }

        let commands = [
            "exit",
            "FC = ",
            "FD = ",
            "FE = ",
            "FF = ",
            "help",
            "quit",
            "ram",
        ];

        let completions = commands.iter().filter_map(|&command| {
            // Only keep commands, for which the input is a real prefix
            if command.starts_with(line) && command != line {
                Some(command.into())
            } else {
                None
            }
        }).collect();

        Ok((0, completions))
    }
}
