use glob::Pattern;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tera::Context;

use crate::errors::{new_error, ErrorKind, Result};
use crate::utils::{read_file, render_one_off_template};
use crate::Value;

/// A condition for a question to be asked
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Condition {
    pub name: String,
    pub value: Value,
}

/// A list of items to be deleted when `name` has `value`
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Cleanup {
    pub name: String,
    pub value: Value,
    pub paths: Vec<String>,
}

/// A question loaded from TOML
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Variable {
    /// The variable name in the final context
    pub name: String,
    /// A default value is required
    pub default: Value,
    /// The text asked to the user
    pub prompt: String,
    /// Only for questions with choices
    pub choices: Option<Vec<Value>>,
    /// A regex pattern to validate the input
    pub validation: Option<String>,
    /// Only ask this variable if that condition is true
    pub only_if: Option<Condition>,
}

/// A hook that should be ran
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Hook {
    /// The display name for that hook
    pub name: String,
    /// The path to the executable file
    pub path: PathBuf,
    /// Only run this hook if that condition is true
    pub only_if: Option<Condition>,
}

/// The full template struct we get fom loading a TOML file
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateDefinition {
    /// Name of the template
    pub name: String,
    /// What this template is about
    pub description: Option<String>,
    /// Template version
    pub version: Option<String>,
    /// Version of the kickstart template spec
    pub kickstart_version: u8,
    /// Url of the template
    pub url: Option<String>,
    /// A list of the authors
    #[serde(default)]
    pub authors: Vec<String>,
    /// Some keywords/tags
    #[serde(default)]
    pub keywords: Vec<String>,
    /// The directory in which the template files are.
    /// Useful if a template has its own docs, README, CI and various files
    pub directory: Option<String>,
    /// Do not copy those directories/files
    #[serde(default)]
    pub ignore: Vec<String>,
    /// Conditionally delete some files/dirs based on generator values
    #[serde(default)]
    pub cleanup: Vec<Cleanup>,
    /// Do not pass those files through Tera. Those can be globs
    #[serde(default)]
    pub copy_without_render: Vec<String>,
    /// Hooks that should be ran after collecting all variables but before generating the template
    #[serde(default)]
    pub pre_gen_hooks: Vec<Hook>,
    /// Hooks that should be ran after generating the template
    #[serde(default)]
    pub post_gen_hooks: Vec<Hook>,
    /// All the questions for that template
    pub variables: Vec<Variable>,
}

impl TemplateDefinition {
    pub(crate) fn all_hooks_paths(&self) -> Vec<String> {
        self.pre_gen_hooks
            .iter()
            .chain(self.post_gen_hooks.iter())
            .map(|h| format!("{}", h.path.display()))
            .collect()
    }

    /// Go through the struct and finds all errors such as invalid globs/regex,
    /// missing/invalid default variable, bad conditions.
    /// If this returns an empty vec, this means the file is valid.
    pub fn validate(&self) -> Vec<String> {
        let mut errs = vec![];
        let mut types = HashMap::new();

        for pattern in &self.copy_without_render {
            if let Err(e) = Pattern::new(pattern) {
                errs.push(format!(
                    "In copy_without_render, `{pattern}` is not a valid pattern: {e}"
                ));
            }
        }

        for hook in self.all_hooks_paths() {
            let p = Path::new(&hook);
            if !p.exists() {
                errs.push(format!("Hook file `{}` was not found", hook));
            }
        }

        for var in &self.variables {
            let type_str = var.default.type_str();
            types.insert(var.name.to_string(), type_str);

            if let Some(ref choices) = var.choices {
                let mut choice_found = false;
                for c in choices {
                    if *c == var.default {
                        choice_found = true;
                    }
                }
                if !choice_found {
                    errs.push(format!(
                        "Variable `{}` has `{}` as default, which isn't in the choices",
                        var.name, var.default
                    ));
                }
            }

            // Since variables are ordered, we can detect whether the only_if is referring
            // to an unknown variable or a variable of the wrong type
            if let Some(ref cond) = var.only_if {
                if let Some(ref t) = types.get(&cond.name) {
                    if **t != cond.value.type_str() {
                        errs.push(format!(
                            "Variable `{}` depends on `{}={}`, but the type of `{}` is {}",
                            var.name, cond.name, cond.value, cond.name, t
                        ));
                    }
                } else {
                    errs.push(format!(
                        "Variable `{}` depends on `{}`, which wasn't asked",
                        var.name, cond.name
                    ));
                }
            }

            if let Some(ref pattern) = var.validation {
                if !var.default.is_str() {
                    errs.push(format!(
                        "Variable `{}` has a validation regex but is not a string",
                        var.name
                    ));
                    continue;
                }

                match Regex::new(pattern) {
                    Ok(re) => {
                        if !re.is_match(var.default.as_str().unwrap()) {
                            errs.push(format!(
                                "Variable `{}` has a default that doesn't pass its validation regex",
                                var.name
                            ));
                        }
                    }
                    Err(_) => {
                        errs.push(format!(
                            "Variable `{}` has an invalid validation regex: {}",
                            var.name, pattern
                        ));
                    }
                }
            }
        }

        errs
    }

    /// Takes a path to a `template.toml` file and validates it.
    /// An Error is only returned if we couldn't load the file or the TOML wasn't valid.
    pub fn validate_file<T: AsRef<Path>>(path: T) -> Result<Vec<String>> {
        let definition: TemplateDefinition = toml::from_str(&read_file(path.as_ref())?)
            .map_err(|err| new_error(ErrorKind::Toml { err }))?;

        Ok(definition.validate())
    }

    /// Returns the default values for all the variables that have one while following conditions
    /// TODO: probably remove that fn? see how to test things
    pub fn default_values(&self) -> Result<HashMap<String, Value>> {
        let mut vals = HashMap::new();
        for var in &self.variables {
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

            match &var.default {
                Value::Boolean(b) => {
                    vals.insert(var.name.clone(), Value::Boolean(*b));
                }
                Value::String(s) => {
                    let mut context = Context::new();
                    for (key, val) in &vals {
                        context.insert(key, val);
                    }
                    let rendered_default = render_one_off_template(s, &context, None)?;
                    vals.insert(var.name.clone(), Value::String(rendered_default));
                }
                Value::Integer(i) => {
                    vals.insert(var.name.clone(), Value::Integer(*i));
                }
            }
        }

        Ok(vals)
    }
}

#[cfg(test)]
mod tests {
    use toml;

    use super::*;

    #[test]
    fn can_validate_definition() {
        insta::glob!("snapshots/validation/*.toml", |path| {
            let errs = TemplateDefinition::validate_file(&path).unwrap();
            insta::assert_debug_snapshot!(&errs);
        });
    }

    #[test]
    fn can_load_template_and_work_with_no_input() {
        let tpl: TemplateDefinition = toml::from_str(
            r#"
            name = "Test template"
            description = "A description"
            kickstart_version = 1

            [[variables]]
            name = "project_name"
            default = "My project"
            prompt = "What's the name of your project?"

            [[variables]]
            name = "database"
            default = "postgres"
            prompt = "Which database to use?"
            choices = ["postgres", "mysql"]

            [[variables]]
            name = "pg_version"
            prompt = "Which version of Postgres?"
            default = "10.4"
            choices = ["10.4", "9.3"]
            only_if = { name = "database", value = "postgres" }

        "#,
        )
        .unwrap();

        assert_eq!(tpl.variables.len(), 3);
        let res = tpl.default_values();
        assert!(res.is_ok());
    }

    #[test]
    fn only_if_questions_are_skipped_if_cond_invalid() {
        let tpl: TemplateDefinition = toml::from_str(
            r#"
            name = "Test template"
            description = "A description"
            kickstart_version = 1

            [[variables]]
            name = "project_name"
            default = "My project"
            prompt = "What's the name of your project?"

            [[variables]]
            name = "database"
            default = "mysql"
            prompt = "Which database to use?"
            choices = ["postgres", "mysql"]

            [[variables]]
            name = "pg_version"
            prompt = "Which version of Postgres?"
            default = "10.4"
            choices = ["10.4", "9.3"]
            only_if = { name = "database", value = "postgres" }

        "#,
        )
        .unwrap();

        assert_eq!(tpl.variables.len(), 3);
        let res = tpl.default_values();
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(!res.contains_key("pg_version"));
    }

    #[test]
    fn nested_only_if_questions_are_skipped_if_initial_cond_invalid() {
        let tpl: TemplateDefinition = toml::from_str(
            r#"
            name = "Test template"
            description = "A description"
            kickstart_version = 1

            [[variables]]
            name = "project_name"
            default = "My project"
            prompt = "What's the name of your project?"

            [[variables]]
            name = "database"
            default = "postgres"
            prompt = "Which database to use?"
            choices = ["postgres", "mysql"]

            [[variables]]
            name = "pg_version"
            prompt = "Which version of Postgres?"
            default = "10.4"
            choices = ["10.4", "9.3"]
            only_if = { name = "database", value = "mysql" }

            [[variables]]
            name = "pg_bouncer"
            prompt = "Add pgBouncer?"
            default = true
            only_if = { name = "pg_version", value = "10.4" }
        "#,
        )
        .unwrap();

        assert_eq!(tpl.variables.len(), 4);
        let res = tpl.default_values();
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(!res.contains_key("pg_version"));
        assert!(!res.contains_key("pg_bouncer"));
    }

    #[test]
    fn use_previous_responses_in_default_value_with_variable_template() {
        let tpl: TemplateDefinition = toml::from_str(
            r#"
            name = "Test template"
            description = "Let's use previous responses to populate default field in other variables"
            kickstart_version = 1

            [[variables]]
            name = "project_one"
            default = "my_project"
            prompt = "What's the name of your first project?"

            [[variables]]
            name = "project_two"
            default = "other_project"
            prompt = "What's the name of your second project?"

            [[variables]]
            name = "manifest"
            default = "{{project_one}}-{{project_two}}-manifest.md"
            prompt = "What's the manifest name file?"
        "#,
        )
        .unwrap();

        assert_eq!(tpl.variables.len(), 3);

        let res = tpl.default_values();

        assert!(res.is_ok());
        let res = res.unwrap();

        assert!(res.contains_key("project_one"));
        assert!(res.contains_key("project_two"));
        assert!(res.contains_key("manifest"));

        let got_value = res.get("manifest").unwrap();
        let expected_value: String = String::from("my_project-other_project-manifest.md");

        assert_eq!(got_value, &Value::String(expected_value))
    }
}
