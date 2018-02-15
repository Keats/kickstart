#[macro_use]
extern crate clap;
extern crate tera;
extern crate toml;
extern crate walkdir;

use std::env;
use std::path::Path;

mod cli;
mod template;
mod prompt;

use template::Template;


fn main() {
    let matches = cli::build_cli().get_matches();
    let template_path = matches.value_of("template").unwrap();
    let output_dir = matches.value_of("output_dir")
        .map(|p| Path::new(p).to_path_buf())
        .unwrap_or_else(|| env::current_dir().unwrap());
    let template = Template::from_cli(template_path);
    template.generate(output_dir);
}
