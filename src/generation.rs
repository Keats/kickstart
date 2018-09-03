use std::env;
use std::path::{Path, PathBuf};
use std::io::Read;
use std::fs::{self, File};
use std::str;
use std::process::Command;

use toml;
use tera::{Tera, Context};
use walkdir::WalkDir;
use glob::Pattern;

use errors::{Result, ErrorKind, new_error};
use utils::{Source, get_source, read_file, write_file, create_directory};
use utils::{is_vcs, is_binary};
use definition::TemplateDefinition;


/// The current template being generated
#[derive(Debug, PartialEq)]
pub struct Template {
    /// Local path to the template folder
    path: PathBuf,
}

impl Template {
    /// Load a template from a string.
    /// It will try to detect whether this is a local folder or whether
    /// it should try to clone it.
    pub fn from_input(input: &str, sub: Option<&str>) -> Result<Template> {
        match get_source(input) {
            Source::Git(remote) => Template::from_git(&remote, sub),
            Source::Local(path) => Ok(Template::from_local(&path, sub)),
        }
    }

    /// Load a template from git.
    /// This will clone the repository if possible in the temporary directory of the user
    pub fn from_git(remote: &str, sub: Option<&str>) -> Result<Template> {
        // Clone the remote in git first in /tmp
        let mut tmp = env::temp_dir();
        tmp.push(remote.split('/').last().unwrap_or_else(|| "kickstart"));
        if tmp.exists() {
            fs::remove_dir_all(&tmp)?;
        }
        println!("Cloning the repository in your temporary folder...");

        // Use git command rather than git2 as it seems there are some issues building it
        // on some platforms:
        // https://www.reddit.com/r/rust/comments/92mbk5/kickstart_a_scaffolding_tool_to_get_new_projects/e3ahegw
        Command::new("git")
            .args(&["clone", remote, &format!("{}", tmp.display())])
            .output()
            .map_err(|err| new_error(ErrorKind::Git { err }))?;

        Ok(Template::from_local(&tmp, sub))
    }

    pub fn from_local(path: &PathBuf, sub: Option<&str>) -> Template {
        let mut buf = path.to_path_buf();
        if let Some(dir) = sub {
            buf.push(dir);
        }
        Template {
            path: buf,
        }
    }

    /// Generate the template at the given output directory
    pub fn generate(&self, output_dir: &PathBuf, no_input: bool) -> Result<()> {
        // Get the variables from the user first
        let conf_path = self.path.join("template.toml");
        if !conf_path.exists() {
            return Err(new_error(ErrorKind::MissingTemplateDefinition));
        }

        let definition: TemplateDefinition = toml::from_str(&read_file(&conf_path)?)
            .map_err(|err| new_error(ErrorKind::Toml { err }))?;

        let variables = definition.ask_questions(no_input)?;
        let mut context = Context::new();
        for (key, val) in &variables {
            context.insert(key, val);
        }

        if !output_dir.exists() {
            create_directory(&output_dir)?;
        }

        // Create the glob patterns of files to copy without rendering first, only once
        let patterns: Vec<Pattern> = definition.copy_without_render
            .iter()
            .map(|s| Pattern::new(s).unwrap())
            .collect();

        // And now generate the files in the output dir given
        let walker = WalkDir::new(&self.path)
            .into_iter()
            .filter_entry(|e| !is_vcs(e))
            .filter_map(|e| e.ok());

        'outer: for entry in walker {
            // Skip root folder and the template.toml
            if entry.path() == self.path || entry.path() == conf_path {
                continue;
            }

            let path = entry.path().strip_prefix(&self.path).unwrap();
            let path_str = format!("{}", path.display());
            for ignored in &definition.ignore {
                if ignored == &path_str || path_str.starts_with(ignored) {
                    continue 'outer;
                }
            }

            let tpl = Tera::one_off(&path_str, &context, false)
                .map_err(|err| new_error(ErrorKind::Tera { err, path: None }))?;

            let real_path = output_dir.join(Path::new(&tpl));

            if entry.path().is_dir() {
                create_directory(&real_path)?;
                continue;
            }

            // Only pass non-binary files or the files not matching the copy_without_render patterns through Tera
            let mut f = File::open(&entry.path())?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;

            let no_render = patterns.iter().map(|p| p.matches_path(&real_path)).any(|x| x);

            if no_render || is_binary(&buffer) {
                fs::copy(&entry.path(), &real_path)
                    .map_err(|err| new_error(ErrorKind::Io { err, path: entry.path().to_path_buf() }))?;
                continue;
            }

            let contents = Tera::one_off(&str::from_utf8(&buffer).unwrap(), &context, false)
                .map_err(|err| new_error(ErrorKind::Tera { err, path: Some(entry.path().to_path_buf()) }))?;
            write_file(&real_path, &contents)?;
        }

        for cleanup in &definition.cleanup {
            if let Some(val) = variables.get(&cleanup.name) {
                if *val == cleanup.value {
                    for p in &cleanup.paths {
                        let actual_path = Tera::one_off(&p, &context, false)
                            .map_err(|err| new_error(ErrorKind::Tera { err, path: None }))?;
                        let path_to_delete = output_dir.join(actual_path);
                        if !path_to_delete.exists() {
                            continue;
                        }
                        if path_to_delete.is_dir() {
                            fs::remove_dir_all(&path_to_delete)
                                .map_err(|err| new_error(ErrorKind::Io { err, path: path_to_delete.to_path_buf() }))?;
                        } else {
                            fs::remove_file(&path_to_delete)
                                .map_err(|err| new_error(ErrorKind::Io { err, path: path_to_delete.to_path_buf() }))?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn can_generate_from_local_path() {
        let dir = tempdir().unwrap();
        let tpl = Template::from_input("examples/complex", None).unwrap();
        let res = tpl.generate(&dir.path().to_path_buf(), true);
        assert!(res.is_ok());
        assert!(!dir.path().join("some-project").join("template.toml").exists());
        assert!(dir.path().join("some-project").join("logo.png").exists());
    }

    #[test]
    fn can_generate_from_remote_repo() {
        let dir = tempdir().unwrap();
        let tpl = Template::from_input("https://github.com/Keats/rust-cli-template", None).unwrap();
        let res = tpl.generate(&dir.path().to_path_buf(), true);
        assert!(res.is_ok());
        assert!(!dir.path().join("My-CLI").join("template.toml").exists());
        assert!(dir.path().join("My-CLI").join(".travis.yml").exists());
    }
}
