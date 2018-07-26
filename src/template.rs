use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use toml_edit::Document;
use tera::{Tera, Context};
use walkdir::WalkDir;

use errors::{Result, ResultExt};
use prompt::{ask_string, ask_bool, ask_choices};
use utils::{Source, get_source, read_file, write_file, create_directory};
use utils::is_vcs;


#[derive(Debug, PartialEq)]
pub struct Template {
    /// Local path to the template folder
    path: PathBuf,
}

impl Template {
    pub fn from_input(input: &str) -> Template {
        match get_source(input) {
            Source::Git(remote) => Template::from_git(&remote),
            Source::Local(path) => Template::from_local(&path),
        }
    }

    pub fn from_git(remote: &str) -> Template {
        // Clone the remote in git first in /tmp
        let mut tmp = env::temp_dir();
        // TODO: generate name from remote
        let repo_name = "kickstart-tmp";
        println!("Cloning the repository in your temporary folder...");

        Command::new("git")
            .current_dir(&tmp)
            .args(&["clone", remote, repo_name])
            .output()
            .expect("Git didn't work, add error handling");

        tmp.push(repo_name);

        Template::from_local(&tmp)
    }

    pub fn from_local(path: &PathBuf) -> Template {
        Template {
            path: path.to_path_buf(),
        }
    }

    fn ask_questions(&self, conf: &Document) -> Result<Context> {
        let table = conf.as_table();
        let mut context = Context::new();

        for (key, data) in table.iter() {
            let var = data.as_table().unwrap();
            // TODO: print invalid questions?
            if let Some(ref question) = var["question"].as_str() {
                if let Some(c) = var.get("choices") {
                    if let Some(default) = var["default"].as_str() {
                        let res = ask_choices(
                            question,
                            default,
                            c.as_array().unwrap(),
                        )?;
                        context.add(key, &res);
                        continue;
                    } else {
                        // TODO print about wrong default for a choice question
                        continue;
                    }
                }

                if let Some(b) = var["default"].as_bool() {
                    let res = ask_bool(question, b)?;
                    context.add(key, &res);
                    continue;
                } else if let Some(s) = var["default"].as_str() {
                    let res = ask_string(question, s)?;
                    context.add(key, &res);
                } else {
                    // TODO: print unknown question type
                }
            }
        }

        Ok(context)
    }

    pub fn generate(&self, output_dir: &PathBuf) -> Result<()> {
        // Get the variables from the user first
        let conf_path = self.path.join("template.toml");
        if !conf_path.exists() {
            bail!("template.toml is missing: is this not a kickstart template?");
        }
        let conf: Document = match read_file(&conf_path)?.parse::<Document>() {
            Ok(d) => d,
            Err(e) => bail!("The template.toml is not valid TOML: {}", e),
        };
        let context = self.ask_questions(&conf)?;

        if !output_dir.exists() {
            create_directory(&output_dir)?;
        }

        // And now generate the files in the output dir given
        let walker = WalkDir::new(&self.path)
            .into_iter()
            .filter_entry(|e| !is_vcs(e))
            .filter_map(|e| e.ok());

        for entry in walker {
            // Skip root folder and the template.toml
            if entry.path() == self.path || entry.path() == conf_path {
                continue;
            }

            let path = entry.path().strip_prefix(&self.path).unwrap();

            let tpl = Tera::one_off(&format!("{}", path.display()), &context, false)
                .chain_err(|| format!("Failed to render {}", path.display()))?;
            let real_path = Path::new(&tpl);

            if entry.path().is_dir() {
                create_directory(&output_dir.join(real_path))?;
            } else {
                let contents = Tera::one_off(&read_file(&entry.path())?, &context, false)
                    .chain_err(|| format!("Failed to render {}", path.display()))?;
                write_file(&output_dir.join(real_path), &contents)?;
            }
        }

        println!("Everything done, ready to go!");

        Ok(())
    }
}
