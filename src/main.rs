#[macro_use]
extern crate clap;
extern crate tera;
extern crate walkdir;
extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate git2;
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
pub mod template;
pub mod errors;

use template::Template;
use errors::{Error, ErrorKind};

fn bail(e: Error) -> ! {
    // Special handling for Tera error-chain
    match e.kind() {
        ErrorKind::Tera {ref err, ..} => {
            print::error(&format!("{}", e));
            for e in err.iter().skip(1) {
                print::error(&format!("{}", e));
            }
        },
        _ => print::error(&format!("{}", e))
    };
    ::std::process::exit(1);
}


fn main() {
    let matches = cli::build_cli().get_matches();
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
