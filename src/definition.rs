use std::collections::HashMap;

use toml::Value;

use prompt::{ask_string, ask_bool, ask_choices, ask_integer};
use errors::{Result, ErrorKind, new_error};


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
                    let res = if no_input {
                        *b
                    } else {
                        ask_bool(&var.prompt, *b)?
                    };
                    vals.insert(var.name.clone(), Value::Boolean(res));
                    continue;
                },
                Value::String(s) => {
                    let res = if no_input {
                        s.clone()
                    } else {
                        ask_string(&var.prompt, &s, &var.validation)?
                    };
                    vals.insert(var.name.clone(), Value::String(res));
                    continue;
                },
                Value::Integer(i) => {
                    let res = if no_input {
                        *i
                    } else {
                        ask_integer(&var.prompt, *i)?
                    };
                    vals.insert(var.name.clone(), Value::Integer(res));
                    continue;
                },
                _ => return Err(new_error(ErrorKind::InvalidTemplate)),
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
    fn can_load_template_and_work_with_no_input() {
        let tpl: TemplateDefinition = toml::from_str(r#"
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

        "#).unwrap();

        assert_eq!(tpl.variables.len(), 3);
        let res = tpl.ask_questions(true);
        assert!(res.is_ok());
    }

    #[test]
    fn only_if_questions_are_skipped_if_cond_invalid() {
        let tpl: TemplateDefinition = toml::from_str(r#"
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

        "#).unwrap();

        assert_eq!(tpl.variables.len(), 3);
        let res = tpl.ask_questions(true);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(!res.contains_key("pg_version"));
    }

    #[test]
    fn nested_only_if_questions_are_skipped_if_initial_cond_invalid() {
        let tpl: TemplateDefinition = toml::from_str(r#"
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
        "#).unwrap();

        assert_eq!(tpl.variables.len(), 4);
        let res = tpl.ask_questions(true);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(!res.contains_key("pg_version"));
        assert!(!res.contains_key("pg_bouncer"));
    }
}
