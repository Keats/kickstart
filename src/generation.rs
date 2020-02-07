use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

use glob::Pattern;
use tera::{Context, Tera};
use toml;
use walkdir::WalkDir;

use crate::definition::TemplateDefinition;
use crate::errors::{new_error, ErrorKind, Result};
use crate::utils::{create_directory, get_source, is_binary, read_file, write_file, Source};

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
    pub fn from_input(input: &str, sub_dir: Option<&str>) -> Result<Template> {
        match get_source(input) {
            Source::Git(remote) => Template::from_git(&remote, sub_dir),
            Source::Local(path) => Ok(Template::from_local(&path, sub_dir)),
        }
    }

    /// Load a template from git.
    /// This will clone the repository if possible in the temporary directory of the user
    pub fn from_git(remote: &str, sub_dir: Option<&str>) -> Result<Template> {
        // Clone the remote in git first in /tmp
        let mut tmp = env::temp_dir();
        println!("Tmp dir: {:?}", tmp);
        tmp.push(remote.split('/').last().unwrap_or_else(|| "kickstart"));
        if tmp.exists() {
            fs::remove_dir_all(&tmp)?;
        }

        // Use git command rather than git2 as it seems there are some issues building it
        // on some platforms:
        // https://www.reddit.com/r/rust/comments/92mbk5/kickstart_a_scaffolding_tool_to_get_new_projects/e3ahegw
        Command::new("git")
            .args(&["clone", remote, &format!("{}", tmp.display())])
            .output()
            .map_err(|err| new_error(ErrorKind::Git { err }))?;
        Ok(Template::from_local(&tmp, sub_dir))
    }

    pub fn from_local(path: &PathBuf, sub_dir: Option<&str>) -> Template {
        let mut buf = path.to_path_buf();
        if let Some(dir) = sub_dir {
            buf.push(dir);
        }
        Template { path: buf }
    }

    fn render_template(&self, content: &str, context: &Context, path: Option<PathBuf>) -> Result<String> {
        let mut tera = Tera::default();

        tera.add_raw_template("one_off", content)
            .and_then(|_| tera.render("one_off", context))
            .map_err(|err| new_error(ErrorKind::Tera { err, path }))
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
        let patterns: Vec<Pattern> =
            definition.copy_without_render.iter().map(|s| Pattern::new(s).unwrap()).collect();

        let start_path = if let Some(ref directory) = definition.directory {
            self.path.join(&directory)
        } else {
            self.path.clone()
        };

        // And now generate the files in the output dir given
        let walker = WalkDir::new(&start_path)
            .into_iter()
            .filter_entry(|e| {
                // Ignore .git/ folder
                let relative_path = e.path().strip_prefix(&start_path).expect("Stripping prefix");
                if relative_path.starts_with(".git/")
                    || (relative_path.is_dir() && relative_path.starts_with(".git"))
                {
                    return false;
                }
                true
            })
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

            let path_str = path_str.replace("$$", "|");

            let tpl = self.render_template(&path_str, &context, None)?;

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
                fs::copy(&entry.path(), &real_path).map_err(|err| {
                    new_error(ErrorKind::Io { err, path: entry.path().to_path_buf() })
                })?;
                continue;
            }

            let contents = self.render_template(&str::from_utf8(&buffer).unwrap(),
                                                &context, Some(entry.path().to_path_buf()))?;

            write_file(&real_path, &contents)?;
        }

        for cleanup in &definition.cleanup {
            if let Some(val) = variables.get(&cleanup.name) {
                if *val == cleanup.value {
                    for p in &cleanup.paths {
                        let actual_path = self.render_template(&p, &context, None)?;
                        let path_to_delete = output_dir.join(actual_path);
                        if !path_to_delete.exists() {
                            continue;
                        }
                        if path_to_delete.is_dir() {
                            fs::remove_dir_all(&path_to_delete).map_err(|err| {
                                new_error(ErrorKind::Io { err, path: path_to_delete.to_path_buf() })
                            })?;
                        } else {
                            fs::remove_file(&path_to_delete).map_err(|err| {
                                new_error(ErrorKind::Io { err, path: path_to_delete.to_path_buf() })
                            })?;
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
    fn can_generate_from_local_path_with_directory() {
        let dir = tempdir().unwrap();
        let tpl = Template::from_input("examples/with-directory", None).unwrap();
        let res = tpl.generate(&dir.path().to_path_buf(), true);
        assert!(res.is_ok());
        assert!(dir.path().join("Hello").join("Howdy.py").exists());
    }

    #[test]
    fn can_generate_from_local_path_with_subdir() {
        let dir = tempdir().unwrap();
        let tpl = Template::from_input("./", Some("examples/complex")).unwrap();
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
        println!("{:?}", res);
        assert!(res.is_ok());
        assert!(!dir.path().join("My-CLI").join("template.toml").exists());
        assert!(dir.path().join("My-CLI").join(".travis.yml").exists());
    }

    #[test]
    fn can_generate_from_remote_repo_with_subdir() {
        let dir = tempdir().unwrap();
        let tpl =
            Template::from_input("https://github.com/Keats/kickstart", Some("examples/complex"))
                .unwrap();
        let res = tpl.generate(&dir.path().to_path_buf(), true);
        println!("{:?}", res);
        assert!(res.is_ok());
        assert!(!dir.path().join("some-project").join("template.toml").exists());
        assert!(dir.path().join("some-project").join("logo.png").exists());
    }

    #[test]
    fn can_generate_handling_slugify() {
        let dir = tempdir().unwrap();
        let tpl = Template::from_input("examples/slugify", None).unwrap();
        let res = tpl.generate(&dir.path().to_path_buf(), true);
        assert!(res.is_ok());
        assert!(!dir.path().join("template.toml").exists());
        assert!(dir.path().join("hello.md").exists());
    }
}
