#[macro_use]
extern crate clap;
extern crate tera;
extern crate walkdir;
extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate memchr;
extern crate glob;
extern crate regex;
extern crate term;

use std::env;
use std::path::Path;

mod cli;
mod definition;
mod prompt;
mod utils;
mod print;
mod validate;
pub mod template;
pub mod errors;

use template::Template;
use errors::{Error, ErrorKind};
use validate::validate_file;


fn bail(e: Error) -> ! {
    // Special handling for Tera error-chain
    match e.kind() {
        ErrorKind::Tera {ref err, ..} => {
            print::error(&format!("{}\n", e));
            for e in err.iter().skip(1) {
                print::error(&format!("{}\n", e));
            }
        },
        _ => print::error(&format!("{}", e))
    };
    ::std::process::exit(1);
}


fn main() {
    let matches = cli::build_cli().get_matches();

    match matches.subcommand() {
        ("validate", Some(matches)) => {
            let errs = match validate_file(matches.value_of("path").unwrap()) {
                Ok(e) => e,
                Err(e) =>  bail(e),
            };

            if !errs.is_empty() {
                print::error("The template.toml is invalid:\n");
                for err in errs {
                    print::error(&format!("- {}\n", err));
                }
                ::std::process::exit(1);
            } else {
                print::success("\nThe template.toml file is valid!\n");
            }

        },
        _ => {
            // The actual generation call
            let template_path = matches.value_of("template").unwrap();
            let output_dir = matches.value_of("output-dir")
                .map(|p| Path::new(p).to_path_buf())
                .unwrap_or_else(|| env::current_dir().unwrap());

            let template = match Template::from_input(template_path) {
                Ok(t) => t,
                Err(e) => bail(e),
            };

            match template.generate(&output_dir) {
                Ok(_) => print::success("\nEverything done, ready to go!\n"),
                Err(e) => bail(e),
            };
        }
    }

}
