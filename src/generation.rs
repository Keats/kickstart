use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

use glob::Pattern;
use tempfile::{tempdir, TempDir};
use tera::Context;
use walkdir::WalkDir;

use crate::definition::{Hook, TemplateDefinition};
use crate::errors::{map_io_err, new_error, ErrorKind, Result};
use crate::utils::{
    create_directory, get_source, is_binary, read_file, render_one_off_template, write_file, Source,
};
use crate::{Value, Variable};

/// Contains information about a given hook: what's the original path and what's the path
/// to the templated version
#[derive(Debug)]
pub struct HookFile {
    hook: Hook,
    /// Canonical path to the hook file after templating
    path: PathBuf,
}

impl HookFile {
    pub fn name(&self) -> &str {
        &self.hook.name
    }
    /// The hook original path in the template folder
    pub fn original_path(&self) -> &Path {
        &self.hook.path
    }

    /// The rendered hook canonicalized file path, this is what you want to execute.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// The current template being generated
#[derive(Debug)]
pub struct Template {
    /// The parsed template definition
    pub definition: TemplateDefinition,
    /// The variables set by the user, either interactively or through the library
    variables: HashMap<String, Value>,
    /// Local path to the template folder
    path: PathBuf,
    /// Temp dir created to store the hooks after templating
    tmp_dir: TempDir,
}

impl Template {
    /// Load a template from a string.
    /// It will try to detect whether this is a local folder or whether
    /// it should try to clone it.
    pub fn from_input(input: &str, directory: Option<&str>) -> Result<Template> {
        match get_source(input) {
            Source::Git(remote) => Template::from_git(&remote, directory),
            Source::Local(path) => Template::from_local(&path, directory),
        }
    }

    /// Load a template from git.
    /// This will clone the repository if possible in the temporary directory of the user
    pub fn from_git(remote: &str, directory: Option<&str>) -> Result<Template> {
        // Clone the remote in git first in /tmp
        let mut tmp = env::temp_dir();
        tmp.push(remote.split('/').last().unwrap_or("kickstart"));
        if tmp.exists() {
            fs::remove_dir_all(&tmp)?;
        }

        // Use git command rather than git2 as it seems there are some issues building it
        // on some platforms:
        // https://www.reddit.com/r/rust/comments/92mbk5/kickstart_a_scaffolding_tool_to_get_new_projects/e3ahegw
        Command::new("git")
            .args(["clone", "--recurse-submodules", remote, &format!("{}", tmp.display())])
            .output()
            .map_err(|err| new_error(ErrorKind::Git { err }))?;
        Template::from_local(&tmp, directory)
    }

    pub fn from_local(path: &Path, directory: Option<&str>) -> Result<Template> {
        let mut buf = path.to_path_buf();
        if let Some(dir) = directory {
            buf.push(dir);
        }
        let conf_path = buf.join("template.toml");
        if !conf_path.exists() {
            return Err(new_error(ErrorKind::MissingTemplateDefinition));
        }

        let definition: TemplateDefinition = toml::from_str(&read_file(&conf_path)?)
            .map_err(|err| new_error(ErrorKind::Toml { err }))?;

        Ok(Template { path: buf, definition, variables: HashMap::new(), tmp_dir: tempdir()? })
    }

    fn get_variable_by_name(&self, name: &str) -> Result<&Variable> {
        if let Some(var) = self.definition.variables.iter().find(|v| v.name == name) {
            Ok(var)
        } else {
            Err(new_error(ErrorKind::InvalidVariableName(name.to_string())))
        }
    }

    pub fn get_default_for(&self, name: &str, vals: &HashMap<String, Value>) -> Result<Value> {
        let var = self.get_variable_by_name(name)?;
        match &var.default {
            Value::Integer(i) => Ok(Value::Integer(*i)),
            Value::Boolean(i) => Ok(Value::Boolean(*i)),
            Value::String(i) => {
                // TODO: Very inefficient but might be ok?
                let mut context = Context::new();
                for (key, val) in vals {
                    context.insert(key, val);
                }
                let rendered_default = render_one_off_template(i, &context, None)?;
                Ok(Value::String(rendered_default))
            }
        }
    }

    pub fn insert_variable(&mut self, name: &str, value: Value) -> Result<()> {
        self.get_variable_by_name(name)?;
        self.variables.insert(name.to_string(), value);

        Ok(())
    }

    /// Overwrites the variables for the template
    pub fn set_variables(&mut self, variables: HashMap<String, Value>) -> Result<()> {
        self.variables.clear();
        for (name, val) in variables {
            self.insert_variable(&name, val)?;
        }
        Ok(())
    }

    fn get_hooks(&self, hooks: &[Hook]) -> Result<Vec<HookFile>> {
        let mut context = Context::new();
        for (key, val) in &self.variables {
            context.insert(key, val);
        }

        let mut hooks_files = Vec::new();

        for hook in hooks {
            // First we check whether we need to run it or not
            if let Some(cond) = &hook.only_if {
                if let Some(val) = self.variables.get(&cond.name) {
                    if *val != cond.value {
                        continue;
                    }
                } else {
                    // Not having it means we didn't even ask the question
                    continue;
                }
            }

            // Then we will read the content of the file and run it through Tera
            let content = read_file(&self.path.join(&hook.path))?;
            let rendered = render_one_off_template(&content, &context, Some(hook.path.clone()))?;

            // Then we save it in a temporary file
            let out_hook_path =
                self.tmp_dir.path().join(hook.path.file_name().expect("to have a filename"));
            let mut file = File::create(&out_hook_path)?;
            write!(file, "{}", rendered)?;
            // TODO: how to make it work for windows
            #[cfg(unix)]
            {
                fs::set_permissions(&out_hook_path, fs::Permissions::from_mode(0o755))?;
            }
            hooks_files.push(HookFile { path: out_hook_path, hook: hook.clone() });
        }

        Ok(hooks_files)
    }

    /// Returns the paths of the hooks that need to be ran in the pre-gen step.
    /// The path will point to a temporary file and not the path of the template as it will
    /// be templated.
    pub fn get_pre_gen_hooks(&self) -> Result<Vec<HookFile>> {
        self.get_hooks(&self.definition.pre_gen_hooks)
    }

    /// Returns the paths of the hooks that need to be ran in the post-gen step.
    /// The path will point to a temporary file and not the path of the template as it will
    /// be templated.
    pub fn get_post_gen_hooks(&self) -> Result<Vec<HookFile>> {
        self.get_hooks(&self.definition.post_gen_hooks)
    }

    pub fn should_ask_variable(&self, name: &str) -> Result<bool> {
        let var = self.get_variable_by_name(name)?;
        if let Some(ref cond) = var.only_if {
            if let Some(val) = self.variables.get(&cond.name) {
                Ok(val == &cond.value)
            } else {
                // This means we never even asked the question
                Ok(false)
            }
        } else {
            Ok(true)
        }
    }

    /// Generate the template at the given output directory
    pub fn generate(&self, output_dir: &Path) -> Result<()> {
        let mut context = Context::new();
        for (key, val) in &self.variables {
            context.insert(key, val);
        }

        if !output_dir.exists() {
            create_directory(output_dir)?;
        }
        let output_dir = output_dir.canonicalize()?;

        // Create the glob patterns of files to copy without rendering first, only once
        let mut patterns = Vec::with_capacity(self.definition.copy_without_render.len());
        for s in &self.definition.copy_without_render {
            let rendered = render_one_off_template(s, &context, None)?;
            match Pattern::new(&rendered) {
                Ok(p) => patterns.push(p),
                Err(err) => {
                    return Err(new_error(ErrorKind::InvalidGlobPattern {
                        err,
                        pattern_before_rendering: s.clone(),
                        pattern_after_rendering: if s == &rendered { None } else { Some(rendered) },
                    }));
                }
            };
        }

        let start_path = if let Some(ref directory) = self.definition.directory {
            self.path.join(directory)
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
                    || e.path().canonicalize().expect("to canonicalize").starts_with(&output_dir)
                {
                    return false;
                }
                true
            })
            .filter_map(|e| e.ok());

        let hooks_paths = self.definition.all_hooks_paths();

        'outer: for entry in walker {
            // Skip root folder and the template.toml
            if entry.path() == self.path || entry.path() == self.path.join("template.toml") {
                continue;
            }

            let path = entry.path().strip_prefix(&self.path).unwrap();
            if path.starts_with(&output_dir) {
                continue;
            }
            let path_str = format!("{}", path.display());
            for ignored in &self.definition.ignore {
                if ignored == &path_str || path_str.starts_with(ignored) {
                    continue 'outer;
                }
            }

            // We automatically ignore hooks file
            if hooks_paths.contains(&path_str) {
                continue 'outer;
            }

            let path_str = path_str.replace("$$", "|");
            let tpl = render_one_off_template(&path_str, &context, None)?;
            let real_path = output_dir.join(Path::new(&tpl));

            if entry.path().is_dir() {
                create_directory(&real_path)?;
                continue;
            }

            // Only pass non-binary files or the files not matching the copy_without_render patterns through Tera
            let mut f = File::open(entry.path())?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;

            // For patterns, we do not want the output directory to be included
            let glob_real_path = real_path.strip_prefix(&output_dir).expect("valid path");
            let no_render = patterns.iter().map(|p| p.matches_path(glob_real_path)).any(|x| x);

            if no_render || is_binary(&buffer) {
                map_io_err(fs::copy(entry.path(), &real_path), entry.path())?;
                continue;
            }

            let contents = render_one_off_template(
                str::from_utf8(&buffer).unwrap(),
                &context,
                Some(entry.path().to_path_buf()),
            )?;

            write_file(&real_path, &contents)?;
        }

        for cleanup in &self.definition.cleanup {
            if let Some(val) = self.variables.get(&cleanup.name) {
                if *val == cleanup.value {
                    for p in &cleanup.paths {
                        let actual_path = render_one_off_template(p, &context, None)?;
                        let path_to_delete = output_dir.join(actual_path).canonicalize()?;
                        // Avoid path traversals
                        if !path_to_delete.starts_with(&output_dir) || !path_to_delete.exists() {
                            continue;
                        }
                        if path_to_delete.is_dir() {
                            map_io_err(fs::remove_dir_all(&path_to_delete), &path_to_delete)?;
                        } else {
                            map_io_err(fs::remove_file(&path_to_delete), &path_to_delete)?;
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
        let mut tpl = Template::from_input("examples/complex", None).unwrap();
        tpl.set_variables(tpl.definition.default_values().unwrap());
        let res = tpl.generate(&dir.path().to_path_buf());

        assert!(res.is_ok());
        assert!(!dir.path().join("some-project").join("template.toml").exists());
        assert!(dir.path().join("some-project").join("logo.png").exists());
    }

    #[test]
    fn can_generate_from_local_path_with_directory() {
        let dir = tempdir().unwrap();
        let mut tpl = Template::from_input("examples/with-directory", None).unwrap();
        tpl.set_variables(tpl.definition.default_values().unwrap());
        let res = tpl.generate(&dir.path().to_path_buf());
        assert!(res.is_ok());
        assert!(dir.path().join("template_root").join("Howdy.py").exists());
    }

    #[test]
    fn can_generate_from_local_path_with_directory_param() {
        let dir = tempdir().unwrap();
        let mut tpl = Template::from_input("./", Some("examples/complex")).unwrap();
        tpl.set_variables(tpl.definition.default_values().unwrap());
        let res = tpl.generate(&dir.path().to_path_buf());
        assert!(res.is_ok());
        assert!(!dir.path().join("some-project").join("template.toml").exists());
        assert!(dir.path().join("some-project").join("logo.png").exists());
    }

    #[test]
    fn can_generate_from_remote_repo() {
        let dir = tempdir().unwrap();
        let mut tpl =
            Template::from_input("https://github.com/Keats/rust-cli-template", None).unwrap();
        tpl.set_variables(tpl.definition.default_values().unwrap());
        let res = tpl.generate(&dir.path().to_path_buf());

        assert!(res.is_ok());
        assert!(!dir.path().join("My-CLI").join("template.toml").exists());
        assert!(dir.path().join("My-CLI").join(".travis.yml").exists());
    }

    #[test]
    fn can_generate_from_remote_repo_with_directory() {
        let dir = tempdir().unwrap();
        let mut tpl =
            Template::from_input("https://github.com/Keats/kickstart", Some("examples/complex"))
                .unwrap();
        tpl.set_variables(tpl.definition.default_values().unwrap());
        let res = tpl.generate(&dir.path().to_path_buf());

        assert!(res.is_ok());
        assert!(!dir.path().join("some-project").join("template.toml").exists());
        assert!(dir.path().join("some-project").join("logo.png").exists());
    }

    #[test]
    fn can_generate_handling_slugify() {
        let dir = tempdir().unwrap();
        let mut tpl = Template::from_input("examples/slugify", None).unwrap();
        tpl.set_variables(tpl.definition.default_values().unwrap());
        let res = tpl.generate(&dir.path().to_path_buf());
        assert!(res.is_ok());
        assert!(!dir.path().join("template.toml").exists());
        assert!(dir.path().join("hello.md").exists());
    }
}
