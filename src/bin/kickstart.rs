use std::env;
use std::error::Error;
use std::path::Path;

use clap::{arg, command, Command};

use kickstart::generation::Template;
use kickstart::terminal;
use kickstart::validate::validate_file;

pub fn build_cli() -> Command {
    command!()
        .arg(arg!("kickstart"))
        // .about(crate_description!())
        // .setting(AppSettings::SubcommandsNegateReqs)
        .arg(
            arg!([name] "template")
                .required(true)
                .help("Template to use: a local path or a HTTP url pointing to a Git repository"),
        )
        .arg(
            arg!([name] "output-dir")
                .short('o')
                .long("output-dir")
                .num_args(1)
                .help("Where to output the project: defaults to the current directory"),
        )
        .arg(
            arg!([name]  "sub-dir")
                .short('s')
                .long("sub-dir")
                .num_args(1)
                .help("A subdirectory of the chosen template to use, to allow nested templates."),
        )
        .arg(
            arg!([name] "no-input")
                .long("no-input")
                .help("Do not prompt for parameters and only use the defaults from template.toml"),
        )
        .subcommand(
            Command::new("validate")
                .about("Validates that a template.toml is valid")
                .arg(arg!([name] "path").required(true).help("The path to the template.toml")),
        )
}

fn bail(e: &dyn Error) -> ! {
    terminal::error(&format!("Error: {e}"));
    let mut cause = e.source();
    while let Some(e) = cause {
        terminal::error(&format!("Reason: {e}"));
        cause = e.source();
    }
    ::std::process::exit(1)
}

fn main() {
    let matches = build_cli().get_matches();

    match matches.subcommand() {
        Some(("validate", sub_matches)) => {
            let errs = match validate_file(sub_matches.get_one::<String>("path").unwrap()) {
                Ok(e) => e,
                Err(e) => bail(&e),
            };

            if !errs.is_empty() {
                terminal::error("The template.toml is invalid:\n");
                for err in errs {
                    terminal::error(&format!("- {err}\n"));
                }
                ::std::process::exit(1);
            } else {
                terminal::success("The template.toml file is valid!\n");
            }
        }
        _ => {
            // The actual generation call
            let template_path = matches.get_one::<String>("template").unwrap();
            let output_dir = matches
                .get_one::<String>("output-dir")
                .map(|p| Path::new(p).to_path_buf())
                .unwrap_or_else(|| env::current_dir().unwrap());
            let no_input = matches.get_one::<String>("no-input").is_some();
            let sub_dir = matches.get_one::<String>("sub-dir");

            let template = match Template::from_input(template_path, sub_dir.map(|x| &**x)) {
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
