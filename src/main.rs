#[macro_use]
extern crate clap;
extern crate tera;
extern crate walkdir;
extern crate toml_edit;
#[macro_use]
extern crate error_chain;

use std::env;
use std::path::Path;

mod cli;
mod template;
mod prompt;
mod utils;
mod errors;

use template::Template;
use errors::Error;

pub fn unravel_errors(error: &Error) {
    println!("Error: {}", error);
    for e in error.iter().skip(1) {
        println!("Reason: {}", e);
    }
}


fn main() {
    let matches = cli::build_cli().get_matches();
    let template_path = matches.value_of("template").unwrap();
    let output_dir = matches.value_of("output-dir")
        .map(|p| Path::new(p).to_path_buf())
        .unwrap_or_else(|| env::current_dir().unwrap());

    let template = Template::from_input(template_path);

    match template.generate(&output_dir) {
        Ok(_) => (),
        Err(e) => {
            println!("Failed to generate template");
            unravel_errors(&e);
            ::std::process::exit(1);
        }
    }
}
