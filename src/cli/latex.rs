use std::borrow::Cow;
use std::fs::File;
use std::path::Path;

use clap::ArgMatches;
use emulator::Instruction;
use emulator::parse::read_reachable_program;

static TEMPLATE: &'static str = include_str!("latex.tex");

pub fn main(args: &ArgMatches<'_>) -> Result<(), i32> {
    // Load programs eagerly and remember their paths
    let programs = args.values_of("2i-programm").unwrap().map(|arg| {
        let program_path = Path::new(arg);
        let program_file = File::open(program_path).map_err(|e| {
            eprintln!("Die angegebene Datei konnte nicht geöffnet werden: {}", e);
            2
        })?;
        let program = read_reachable_program(&program_file).map_err(|e| {
            eprintln!("Das Mikroprogramm konnte nicht geladen werden: {}", e);
            3
        })?;

        Ok((program_path.to_owned(), program))
    }).collect::<Result<Vec<_>,i32>>()?;

    // Load and split template
    let mut template = TEMPLATE.split("#split#");

    // Print the header of the latex template
    let author = if let Some(authors) = args.values_of("autor") {
        let prefix = "\\indent -- ".to_owned();
        Cow::Owned(prefix + &authors.collect::<Vec<_>>().join(", "))
    } else {
        "".into()
    };
    let header = template.next().unwrap();
    print!("{}", header.replace("#author#", &author));

    let page_separator = template.next().unwrap();

    // The first page can only contain 37 lines because of the header
    let mut lines_remaining = 37;
    for (path, program) in programs {
        // 2 lines are used for the program header and some margin
        if lines_remaining < program.len() + 2 {
            // Start new program table on new page (works because programs
            // cannot be longer than 32 + 2 lines)
            print!("{}", page_separator);
            lines_remaining = 40;
        }
        lines_remaining -= program.len() + 2;

        print_program(&path.to_string_lossy(), &program);
    }

    // Print the footer of the latex template
    print!("{}", template.next().unwrap());

    Ok(())
}

fn print_program(filename: &str, program: &Vec<(u8, Instruction)>) {
    println!();
    println!("    % Generated from {}", filename);
    println!("    \\multicolumn{{15}}{{l}}{{}}\\\\\\multicolumn{{15}}{{l}}{{\\textbf{{{}}}}}\\\\\\hline", escape_latex(filename));

    for &(addr, inst) in program.iter() {
        println!("    {}&\\verb|{}|&{:05b}&{:02b}&{:05b}&{:01b}&{:01b}&{:03b}&{:04b}&{:01b}&{:01b}&{:01b}&{:01b}&{:04b}&{:01b}\\\\\\hline",
            addr,
            inst.to_mnemonic(Some(addr as usize)),
            addr,
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
            inst.should_store_flags() as u8);
    }
}

fn escape_latex(string: &str) -> String {
    let mut result = String::with_capacity(string.len());

    for char in string.chars() {
        match char {
            '&' => result.push_str("\\&"),
            '%' => result.push_str("\\%"),
            '$' => result.push_str("\\$"),
            '#' => result.push_str("\\#"),
            '_' => result.push_str("\\_"),
            '{' => result.push_str("\\{"),
            '}' => result.push_str("\\}"),
            '~' => result.push_str("\\textasciitilde{}"),
            '^' => result.push_str("\\textasciicircum{}"),
            '\\' => result.push_str("\\textbackslash{}"),
            _ => result.push(char),
        }
    }

    result
}
