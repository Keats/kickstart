use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command as StdCommand;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};

use kickstart::cli::file_input::load_json_input;
use kickstart::cli::prompt::{ask_bool, ask_choices, ask_integer, ask_string};
use kickstart::cli::terminal;
use kickstart::{HookFile, Template, TemplateDefinition, Value};

#[derive(Parser)]
#[clap(version, author, about, subcommand_negates_reqs = true)]
pub struct Cli {
    /// Template to use: a local path or a HTTP url pointing to a Git repository
    #[clap(required = true)]
    pub template: Option<String>,

    /// Where to output the project: defaults to the current directory
    #[clap(short = 'o', long, default_value_os_t = PathBuf::from("."))]
    pub output_dir: PathBuf,

    /// The directory of the given folder/repository to use, which needs to be a template.
    /// Only really useful if you are loading a template from a repository. If you are loading
    /// from the filesystem you can directly point to the right folder.
    #[clap(short = 'd', long)]
    pub directory: Option<String>,

    /// Do not prompt for variables and only use the defaults from template.toml
    #[clap(long, default_value_t = false)]
    pub no_input: bool,

    /// Path to a JSON file containing variable values (implies --no-input)
    #[clap(short = 'i', long = "input-file", value_name = "PATH")]
    pub input_file: Option<PathBuf>,

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
/// If a value exists in `provided_values`, it will be validated and used.
/// Otherwise:
/// - if `no_input` is `true`, the default is used without prompting.
/// - else the user is prompted interactively.
fn ask_questions(
    template: &Template,
    no_input: bool,
    provided_values: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>> {
    let mut vals = HashMap::new();

    for var in &template.definition.variables {
        if var.derived.unwrap_or(false) {
            let default = template.get_default_for(&var.name, &vals)?;
            vals.insert(var.name.clone(), default);
            continue;
        }

        if !template.should_ask_variable(&var.name, &vals)? {
            continue;
        }

        let default = template.get_default_for(&var.name, &vals)?;

        // Check if value was provided from JSON input
        if let Some(provided) = provided_values.get(&var.name) {
            vals.insert(var.name.clone(), provided.clone());
            continue;
        }

        // No provided value and input -> use the default
        if no_input {
            vals.insert(var.name.clone(), default);
            continue;
        }

        // Interactive prompting
        let prompt_text = var.prompt.as_deref().unwrap_or("");

        if let Some(ref choices) = var.choices {
            let res = ask_choices(prompt_text, &default, choices)?;
            vals.insert(var.name.clone(), res);
            continue;
        }

        match default {
            Value::Boolean(b) => {
                let res = ask_bool(prompt_text, b)?;
                vals.insert(var.name.clone(), Value::Boolean(res));
            }
            Value::String(s) => {
                let res = ask_string(prompt_text, &s, &var.validation)?;
                vals.insert(var.name.clone(), Value::String(res));
            }
            Value::Integer(i) => {
                let res = ask_integer(prompt_text, i)?;
                vals.insert(var.name.clone(), Value::Integer(res));
            }
        }
    }

    Ok(vals)
}

fn execute_hook(hook: &HookFile, output_dir: &PathBuf) -> Result<()> {
    terminal::bold(&format!("  - {}\n", hook.name()));
    let mut command = StdCommand::new(hook.path());
    if output_dir.exists() {
        command.current_dir(output_dir);
    }
    let code = command.status()?;
    if code.success() { Ok(()) } else { bail!("Hook `{}` exited with a non 0 code\n", hook.name()) }
}

fn try_main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Validate { path }) => {
            let errs = TemplateDefinition::validate_file(path)?;

            if !errs.is_empty() {
                // We let the caller do the error handling/display
                let err = format!(
                    "The template.toml is invalid:\n{}",
                    errs.into_iter().map(|e| format!("- {}\n", e)).collect::<Vec<_>>().join("\n"),
                );
                bail!(err);
            } else {
                terminal::success("The template.toml file is valid!\n");
            }
        }
        None => {
            let mut template =
                Template::from_input(&cli.template.unwrap(), cli.directory.as_deref())?;

            // 1. collect variables (from JSON input or interactive prompts)
            let (no_input, provided_values) = if let Some(ref input_path) = cli.input_file {
                (true, load_json_input(input_path, &template)?)
            } else {
                (cli.no_input, HashMap::new())
            };
            let vals = ask_questions(&template, no_input, &provided_values)?;
            template.set_variables(vals)?;

            // 2. run pre-gen hooks
            let pre_gen_hooks = template.get_pre_gen_hooks()?;
            if cli.run_hooks && !pre_gen_hooks.is_empty() {
                terminal::bold("Running pre-gen hooks...\n");
                for hook in &pre_gen_hooks {
                    execute_hook(hook, &cli.output_dir)?;
                }
                // For spacing
                println!();
            }

            // 3. generate
            template.generate(&cli.output_dir)?;

            // 4. run post-gen hooks
            let post_gen_hooks = template.get_post_gen_hooks()?;
            if cli.run_hooks && !post_gen_hooks.is_empty() {
                terminal::bold("Running post-gen hooks...\n");
                for hook in &post_gen_hooks {
                    execute_hook(hook, &cli.output_dir)?;
                }
                // For spacing
                println!();
            }

            terminal::success("\nEverything done, ready to go!\n");
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = try_main() {
        terminal::error(&format!("Error: {}", e));
        let mut cause = e.source();
        while let Some(e) = cause {
            terminal::error(&format!("\nReason: {}", e));
            cause = e.source();
        }
        ::std::process::exit(1)
    }
}
