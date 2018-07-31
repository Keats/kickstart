use std::collections::HashMap;
use std::path::Path;

use regex::Regex;
use toml;

use errors::{Result, ErrorKind, new_error};
use definition::TemplateDefinition;
use utils::read_file;


/// Validate that the struct doesn't have bad data in it
pub fn validate_definition(def: &TemplateDefinition) -> Vec<String> {
    let mut errs = vec![];
    let mut types = HashMap::new();

    for var in &def.variables {
        types.insert(var.name.to_string(), var.default.type_str());

        if let Some(ref choices) = var.choices {
            let mut choice_found = false;
            for c in choices {
                if *c == var.default {
                    choice_found = true;
                }
            }
            if !choice_found {
                errs.push(
                    format!("Variable `{}` has `{}` as default, which isn't in the choices", var.name, var.default)
                );
            }
        }

        // Since variables are ordered, we can detect whether the only_if is referring
        // to an unknown variable or a variable of the wrong type
        if let Some(ref cond) = var.only_if {
            if let Some(ref t) = types.get(&cond.name) {
                if **t != cond.value.type_str() {
                    errs.push(
                        format!("Variable `{}` depends on `{}={}`, but the type of `{}` is {}", var.name, cond.name, cond.value, cond.name, t)
                    );
                }
            } else {
                errs.push(
                    format!("Variable `{}` depends on `{}`, which wasn't asked", var.name, cond.name)
                );
            }
        }

        if let Some(ref pattern) = var.validation {
            if !var.default.is_str() {
                errs.push(
                    format!("Variable `{}` has a validation regex but is not a string", var.name)
                );
                continue;
            }

            match Regex::new(pattern) {
                Ok(re) => {
                    if !re.is_match(&var.default.as_str().unwrap()) {
                        errs.push(
                            format!("Variable `{}` has a default that doesn't pass its validation regex", var.name)
                        );
                    }
                },
                Err(_) => {
                    errs.push(
                        format!("Variable `{}` has an invalid validation regex: {}", var.name, pattern)
                    );
                }
            }
        }
    }

    errs
}

/// Takes a path to a `template.toml` file and validates it
pub fn validate_file<T: AsRef<Path>>(path: T) -> Result<Vec<String>> {
    let definition: TemplateDefinition = toml::from_str(&read_file(path.as_ref())?)
        .map_err(|err| new_error(ErrorKind::Toml { err }))?;

    Ok(validate_definition(&definition))
}


#[cfg(test)]
mod tests {
    use toml;

    use super::*;

    #[test]
    fn valid_definition_has_no_errors() {
        let def: TemplateDefinition = toml::from_str(r#"
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
        let errs = validate_definition(&def);
        assert!(errs.is_empty());
    }

    #[test]
    fn errors_default_not_in_choice() {
        let def: TemplateDefinition = toml::from_str(r#"
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
            default = "10.5"
            choices = ["10.4", "9.3"]
            only_if = { name = "database", value = "postgres" }

        "#).unwrap();
        let errs = validate_definition(&def);
        assert!(!errs.is_empty());
        assert_eq!(errs[0], "Variable `pg_version` has `\"10.5\"` as default, which isn\'t in the choices");
    }

    #[test]
    fn errors_only_if_unkwnon_variable_name() {
        let def: TemplateDefinition = toml::from_str(r#"
            name = "Test template"
            description = "A description"
            kickstart_version = 1

            [[variables]]
            name = "project_name"
            default = "My project"
            prompt = "What's the name of your project?"

            [[variables]]
            name = "pg_version"
            prompt = "Which version of Postgres?"
            default = "10.4"
            choices = ["10.4", "9.3"]
            only_if = { name = "database", value = true }

        "#).unwrap();
        let errs = validate_definition(&def);
        assert!(!errs.is_empty());
        assert_eq!(errs[0], "Variable `pg_version` depends on `database`, which wasn\'t asked");
    }

    #[test]
    fn errors_only_if_not_matching_type() {
        let def: TemplateDefinition = toml::from_str(r#"
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
            only_if = { name = "database", value = true }

        "#).unwrap();
        let errs = validate_definition(&def);
        assert!(!errs.is_empty());
        assert_eq!(errs[0], "Variable `pg_version` depends on `database=true`, but the type of `database` is string");
    }

    #[test]
    fn errors_validation_regex_on_wrong_type() {
        let def: TemplateDefinition = toml::from_str(r#"
            name = "Test template"
            description = "A description"
            kickstart_version = 1

            [[variables]]
            name = "project"
            default = true
            prompt = "What's the name of your project?"
            validation = "[0-9]+"

        "#).unwrap();
        let errs = validate_definition(&def);
        assert!(!errs.is_empty());
        assert_eq!(errs[0], "Variable `project` has a validation regex but is not a string");
    }

    #[test]
    fn errors_invalid_validation_regex() {
        let def: TemplateDefinition = toml::from_str(r#"
            name = "Test template"
            description = "A description"
            kickstart_version = 1

            [[variables]]
            name = "project_name"
            default = "My project"
            prompt = "What's the name of your project?"
            validation = "**[0-9]++"

        "#).unwrap();
        let errs = validate_definition(&def);
        assert!(!errs.is_empty());
        assert_eq!(errs[0], "Variable `project_name` has an invalid validation regex: **[0-9]++");
    }

    #[test]
    fn errors_default_doesnt_match_validation_regex() {
        let def: TemplateDefinition = toml::from_str(r#"
            name = "Test template"
            description = "A description"
            kickstart_version = 1

            [[variables]]
            name = "project_name"
            default = "123"
            prompt = "What's the name of your project?"
            validation = "^([a-zA-Z][a-zA-Z0-9_-]+)$"

        "#).unwrap();
        let errs = validate_definition(&def);
        assert!(!errs.is_empty());
        assert_eq!(errs[0], "Variable `project_name` has a default that doesn\'t pass its validation regex");
    }
}
