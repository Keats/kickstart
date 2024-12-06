use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::process::Command as StdCommand;

use clap::{Parser, Subcommand};
use tera::Context;
use toml::Value;

use kickstart::errors::{new_error, ErrorKind, Result};
use kickstart::prompt::{ask_bool, ask_choices, ask_integer, ask_string};
use kickstart::utils::render_one_off_template;
use kickstart::validate_file;
use kickstart::Template;
use kickstart::TemplateDefinition;
use kickstart::{terminal, HookFile};

#[derive(Parser)]
#[clap(version, author, about, subcommand_negates_reqs = true)]
pub struct Cli {
    /// Template to use: a local path or a HTTP url pointing to a Git repository
    #[clap(required = true)]
    pub template: Option<String>,

    /// Where to output the project: defaults to the current directory
    #[clap(short = 'o', long, default_value = ".")]
    pub output_dir: PathBuf,

    /// The directory of the given folder/repository to use, which needs to be a template.
    /// Only really useful if you are loading a template from a repository. If you are loading
    /// from the filesystem you can directly point to the right folder.
    #[clap(short = 'd', long)]
    pub directory: Option<String>,

    /// Do not prompt for variables and only use the defaults from template.toml
    #[clap(long, default_value_t = false)]
    pub no_input: bool,

    /// Whether to run the hooks
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
fn ask_questions(
    definition: &TemplateDefinition,
    no_input: bool,
) -> Result<HashMap<String, Value>> {
    let mut vals = HashMap::new();

    for var in &definition.variables {
        // Skip the question if the value is different from the condition
        if let Some(ref cond) = var.only_if {
            if let Some(val) = vals.get(&cond.name) {
                if *val != cond.value {
                    continue;
                }
            } else {
                // Not having it means we didn't even ask the question
                continue;
            }
        }

        if let Some(ref choices) = var.choices {
            let res = if no_input {
                var.default.clone()
            } else {
                ask_choices(&var.prompt, &var.default, choices)?
            };
            vals.insert(var.name.clone(), res);
            continue;
        }

        match &var.default {
            Value::Boolean(b) => {
                let res = if no_input { *b } else { ask_bool(&var.prompt, *b)? };
                vals.insert(var.name.clone(), Value::Boolean(res));
                continue;
            }
            Value::String(s) => {
                let default_value = if s.contains("{{") && s.contains("}}") {
                    let mut context = Context::new();
                    for (key, val) in &vals {
                        context.insert(key, val);
                    }

                    let rendered_default = render_one_off_template(s, &context, None);
                    match rendered_default {
                        Err(e) => return Err(e),
                        Ok(v) => v,
                    }
                } else {
                    s.clone()
                };

                let res = if no_input {
                    default_value
                } else {
                    ask_string(&var.prompt, &default_value, &var.validation)?
                };

                vals.insert(var.name.clone(), Value::String(res));
                continue;
            }
            Value::Integer(i) => {
                let res = if no_input { *i } else { ask_integer(&var.prompt, *i)? };
                vals.insert(var.name.clone(), Value::Integer(res));
                continue;
            }
            _ => return Err(new_error(ErrorKind::InvalidTemplate)),
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
                let err: Box<dyn Error> = format!("Hook `{}` exited with a non 0 code\n", hook.name()).into();
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
            let errs = bail_if_err!(validate_file(path));

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
            let variables = bail_if_err!(ask_questions(&template.definition, cli.no_input));
            template.set_variables(variables);

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
