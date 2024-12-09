use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::process::Command as StdCommand;

use clap::{Parser, Subcommand};

use kickstart::errors::Result;
use kickstart::cli::prompt::{ask_bool, ask_choices, ask_integer, ask_string};
use kickstart::Template;
use kickstart::TemplateDefinition;
use kickstart::{Value, HookFile};
use kickstart::cli::terminal;



#[derive(Parser)]
#[clap(version, author, about, subcommand_negates_reqs = true)]
pub struct Cli {
    /// Template to use: a local path or a HTTP url pointing to a Git repository
    #[clap(required = true)]
    pub template: Option<String>,

    /// Where to output the project: defaults to the current directory
    #[clap(short = 'o', long)]
    pub output_dir: PathBuf,

    /// The directory of the given folder/repository to use, which needs to be a template.
    /// Only really useful if you are loading a template from a repository. If you are loading
    /// from the filesystem you can directly point to the right folder.
    #[clap(short = 'd', long)]
    pub directory: Option<String>,

    /// Do not prompt for variables and only use the defaults from template.toml
    #[clap(long, default_value_t = false)]
    pub no_input: bool,

    /// Whether to run all the hooks
    #[clap(long, default_value_t = true)]
    pub run_hooks: bool,

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

/// Ask all the questions of that template and return the answers.
/// If `no_input` is `true`, it will automatically pick the defaults without
/// prompting the user
fn ask_questions(template: &Template, no_input: bool) -> Result<HashMap<String, Value>> {
    let mut vals = HashMap::new();

    for var in &template.definition.variables {
        if !template.should_ask_variable(&var.name)? {
            continue;
        }
        let default = template.get_default_for(&var.name, &vals)?;

        if let Some(ref choices) = var.choices {
            let res = if no_input { default } else { ask_choices(&var.prompt, &default, choices)? };
            vals.insert(var.name.clone(), res);
            continue;
        }

        match default {
            Value::Boolean(b) => {
                let res = if no_input { b } else { ask_bool(&var.prompt, b)? };
                vals.insert(var.name.clone(), Value::Boolean(res));
                continue;
            }
            Value::String(s) => {
                let res = if no_input { s } else { ask_string(&var.prompt, &s, &var.validation)? };
                vals.insert(var.name.clone(), Value::String(res));
                continue;
            }
            Value::Integer(i) => {
                let res = if no_input { i } else { ask_integer(&var.prompt, i)? };
                vals.insert(var.name.clone(), Value::Integer(res));
                continue;
            }
        }
    }

    Ok(vals)
}

fn bail(e: &dyn Error) -> ! {
    terminal::error(&format!("Error: {}", e));
    let mut cause = e.source();
    while let Some(e) = cause {
        terminal::error(&format!("\nReason: {}", e));
        cause = e.source();
    }
    ::std::process::exit(1)
}

macro_rules! bail_if_err {
    ($expr:expr) => {{
        match $expr {
            Ok(v) => v,
            Err(e) => bail(&e),
        }
    }};
}

fn execute_hook(hook: &HookFile) -> Result<()> {
    terminal::bold(&format!("  - {}\n", hook.name()));
    match StdCommand::new(hook.path()).status() {
        Ok(code) => {
            if code.success() {
                Ok(())
            } else {
                let err: Box<dyn Error> =
                    format!("Hook `{}` exited with a non 0 code\n", hook.name()).into();
                bail(&*err)
            }
        }
        Err(e) => bail(&e),
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Validate { path }) => {
            let errs = bail_if_err!(TemplateDefinition::validate_file(path));

            if !errs.is_empty() {
                terminal::error("The template.toml is invalid:\n");
                for err in errs {
                    terminal::error(&format!("- {}\n", err));
                }
                ::std::process::exit(1);
            } else {
                terminal::success("The template.toml file is valid!\n");
            }
        }
        None => {
            let mut template = bail_if_err!(Template::from_input(
                &cli.template.unwrap(),
                cli.directory.as_deref()
            ));

            // 1. ask questions
            let vals = bail_if_err!(ask_questions(&template, cli.no_input));
            bail_if_err!(template.set_variables(vals));

            // 2. run pre-gen hooks
            let pre_gen_hooks = bail_if_err!(template.get_pre_gen_hooks());
            if cli.run_hooks && !pre_gen_hooks.is_empty() {
                terminal::bold("Running pre-gen hooks...\n");
                for hook in &pre_gen_hooks {
                    bail_if_err!(execute_hook(hook));
                }
                println!();
            }

            // 3. generate
            bail_if_err!(template.generate(&cli.output_dir));

            // 4. run post-gen hooks
            let post_gen_hooks = bail_if_err!(template.get_post_gen_hooks());
            if cli.run_hooks && !post_gen_hooks.is_empty() {
                terminal::bold("Running post-gen hooks...\n");
                for hook in &post_gen_hooks {
                    bail_if_err!(execute_hook(hook));
                }
                println!();
            }

            terminal::success("\nEverything done, ready to go!\n");
        }
    }
}
