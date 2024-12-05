use std::error::Error;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use kickstart::generation::Template;
use kickstart::terminal;
use kickstart::validate::validate_file;

#[derive(Parser)]
#[clap(version, author, about, subcommand_negates_reqs = true)]
pub struct Cli {
    /// Template to use: a local path or a HTTP url pointing to a Git repository
    #[clap(required = true)]
    pub template: Option<String>,

    /// Where to output the project: defaults to the current directory
    #[clap(short = 'o', long, default_value = ".")]
    pub output_dir: PathBuf,

    /// A subdirectory of the chosen template to use, to allow nested templates.
    /// The subdirectory needs to be a template itself.
    #[clap(short = 's', long)]
    pub sub_dir: Option<String>,

    /// Do not prompt for parameters and only use the defaults from template.toml
    #[clap(long, default_value_t = false)]
    pub no_input: bool,

    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Validates that a template.toml is valid
    Validate {
        /// The path to the template.toml
        path: PathBuf,
    },
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
    let cli = Cli::parse();

    if let Some(Command::Validate { path }) = cli.command {
        let errs = match validate_file(path) {
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
            terminal::success("The template.toml file is valid!\n");
        }
    } else {
        let template = match Template::from_input(&cli.template.unwrap(), cli.sub_dir.as_deref()) {
            Ok(t) => t,
            Err(e) => bail(&e),
        };

        match template.generate(&cli.output_dir, cli.no_input) {
            Ok(_) => terminal::success("\nEverything done, ready to go!\n"),
            Err(e) => bail(&e),
        };
    }
}
