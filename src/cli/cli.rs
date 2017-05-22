use std::io;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub fn build() -> App<'static, 'static> {
    App::new("2i-emulator")
        .version(crate_version!())
        .setting(AppSettings::ArgsNegateSubcommands)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::VersionlessSubcommands)
        .set_term_width(80)
        .arg(Arg::with_name("2i-programm")
            .help("Das zu ladende Mikroprogramm"))
        .subcommand(SubCommand::with_name("completions")
            .about("Erstelle Anweisungen zur Autovervollständigung für die angegebene Shell.")
            .setting(AppSettings::Hidden)
            .arg(Arg::with_name("shell")
                .help("bash, fish, zsh, or powershell")
                .required(true)))
        .subcommand(SubCommand::with_name("ipg-csv")
            .about("Konvertiere ein Programm in das ipg-csv-Format, das mit Hilfe von mcontrol auf den Minirechner geladen werden kann.")
            .arg(Arg::with_name("2i-programm")
                .help("Das zu konvertierende Mikroprogramm")
                .required(true)))
        .subcommand(SubCommand::with_name("latex")
            .about("Erstelle ein LaTeX-Dokument mit einer übersichtlichen Darstellung der gegebenen Programme.")
            .arg(Arg::with_name("autor")
                .help("Autoren der Programme")
                .long("autor")
                .number_of_values(1)
                .multiple(true))
            .arg(Arg::with_name("2i-programm")
                .help("Die darzustellenden Programme")
                .required(true)
                .multiple(true)))
}

pub fn gen_completions(args: &ArgMatches) -> Result<(), i32> {
    let shell = args.value_of("shell").unwrap().parse().map_err(|_| {
        println!("Unbekannte Shell: {}", args.value_of("shell").unwrap());
        2
    })?;

    build().gen_completions_to("2i-emulator", shell, &mut io::stdout());

    Ok(())
}
