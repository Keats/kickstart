use std::env;
use std::path::Path;
use std::error::Error;

use clap::{App, Arg, SubCommand, AppSettings, crate_authors, crate_version, crate_description};

use kickstart::terminal;
use kickstart::generation::Template;
use kickstart::validate::validate_file;


pub fn build_cli() -> App<'static, 'static> {
    App::new("kickstart")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandsNegateReqs)
        .arg(
            Arg::with_name("template")
                .required(true)
                .help("Template to use: a local path or a HTTP url pointing to a Git repository")
        )
        .arg(
            Arg::with_name("output-dir")
                .short("o")
                .long("output-dir")
                .takes_value(true)
                .help("Where to output the project: defaults to the current directory")
        )
        .arg(
            Arg::with_name("sub-dir")
                .short("s")
                .long("sub-dir")
                .takes_value(true)
                .help("A subdirectory of the chosen template to use, to allow nested templates.")
        )
        .arg(
            Arg::with_name("no-input")
                .long("no-input")
                .help("Do not prompt for parameters and only use the defaults from template.toml")
        )
        .subcommands(vec![
            SubCommand::with_name("validate")
                .about("Validates that a template.toml is valid")
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .help("The path to the template.toml")
                ),
        ])
}

fn bail(e: &dyn Error) -> ! {
    terminal::error(&format!("Error: {}", e));
    let mut cause = e.source();
    while let Some(e) = cause {
        terminal::error(&format!("Reason: {}", e));
        cause = e.source();
    }
    ::std::process::exit(1)
}


fn main() {
    let matches = build_cli().get_matches();

    match matches.subcommand() {
        ("validate", Some(matches)) => {
            let errs = match validate_file(matches.value_of("path").unwrap()) {
                Ok(e) => e,
                Err(e) => bail(&e),
            };

            if !errs.is_empty() {
                terminal::error("The template.toml is invalid:\n");
                for err in errs {
                    terminal::error(&format!("- {}\n", err));
                }
                ::std::process::exit(1);
            } else {
                terminal::success("\nThe template.toml file is valid!\n");
            }
        }
        _ => {
            // The actual generation call
            let template_path = matches.value_of("template").unwrap();
            let output_dir = matches.value_of("output-dir")
                .map(|p| Path::new(p).to_path_buf())
                .unwrap_or_else(|| env::current_dir().unwrap());
            let no_input = matches.is_present("no-input");
            let sub_dir = matches.value_of("sub-dir");

            let template = match Template::from_input(template_path, sub_dir) {
                Ok(t) => t,
                Err(e) => bail(&e),
            };

            match template.generate(&output_dir, no_input) {
                Ok(_) => terminal::success("\nEverything done, ready to go!\n"),
                Err(e) => bail(&e),
            };
        }
    }
}
