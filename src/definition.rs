use std::collections::{HashMap};

use lazy_static::lazy_static;
use regex::{Regex, Match};
use serde::Deserialize;
use tera::{Context};
use toml::Value;

use crate::errors::{new_error, ErrorKind, Result};
use crate::prompt::{ask_bool, ask_choices, ask_integer, ask_string};
use crate::utils::{render_one_off_template};

/// A condition for a question to be asked
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VariableCondition {
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
    /// Only ask this variable is the condition is true
    pub only_if: Option<VariableCondition>,
}

/// The full template struct we get fom loading a TOML file
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TemplateDefinition {
    /// Name of the template
    pub name: String,
    /// What this template is about
    pub description: Option<String>,
    /// Template version
    pub version: Option<String>,
    /// Version of the kickstart template spec
    pub kickstart_version: usize,
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
    /// Do not pass those files through Tera
    /// http://cookiecutter.readthedocs.io/en/latest/advanced/copy_without_render.html
    #[serde(default)]
    pub copy_without_render: Vec<String>,
    /// All the questions for that template
    pub variables: Vec<Variable>,
}

impl TemplateDefinition {
    /// Ask all the questions of that template and return the answers.
    /// If `no_input` is `true`, it will automatically pick the default without
    /// prompting the user
    pub fn ask_questions(&self, no_input: bool) -> Result<HashMap<String, Value>> {
        // Tera context doesn't expose a way to get value from a context
        // so we store them in another hashmap
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
                    let contains_template = has_template_variables(&s);
                    let default_value = match contains_template {
                        Some(_) => {
                            let mut context = Context::new();
                            for (key, val) in &vals {
                                context.insert(key, val);
                            }

                            let rendered_default = render_one_off_template(&s, &context, None);
                            match rendered_default {
                                Err(e) => return Err(e),
                                Ok(v ) => v,
                            }
                        },
                        None => s.clone(),                 
                    };

                    println!("{:?}", default_value);

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
}

fn has_template_variables<'a>(s: &'a String) -> Option<Match> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\{\{(?:[a-zA-Z][0-9a-zA-Z_]*)\}\}").unwrap();
    }

    RE.find(s)
}

#[cfg(test)]
mod tests {
    use toml;

    use super::*;

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
        let res = tpl.ask_questions(true);
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
            default = "postgres"
            prompt = "Which database to use?"
            choices = ["postgres", "mysql"]

            [[variables]]
            name = "pg_version"
            prompt = "Which version of Postgres?"
            default = "10.4"
            choices = ["10.4", "9.3"]
            only_if = { name = "database", value = "mysql" }

        "#,
        )
        .unwrap();

        assert_eq!(tpl.variables.len(), 3);
        let res = tpl.ask_questions(true);
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
        let res = tpl.ask_questions(true);
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

        let res = tpl.ask_questions(true);
        
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
