use std::collections::HashMap;
use std::path::Path;

use crate::cli::terminal;
use crate::{Template, Value};
use anyhow::{Context, Result, bail};
use regex::Regex;

/// Load and validate variable values from a JSON file
pub fn load_json_input(path: &Path, template: &Template) -> Result<HashMap<String, Value>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read input file: {}", path.display()))?;

    let json: serde_json::Value =
        serde_json::from_str(&content).with_context(|| "Failed to parse JSON input file")?;

    let obj = json.as_object().ok_or_else(|| anyhow::anyhow!("JSON input must be an object"))?;

    let mut values = HashMap::new();
    for (key, json_val) in obj {
        // Check if this variable exists in the template
        let var = match template.get_variable_by_name(key) {
            Ok(v) => v,
            Err(_) => {
                terminal::error(&format!(
                    "Warning: Unknown variable '{}' in JSON input (ignored)\n",
                    key
                ));
                continue;
            }
        };

        // Convert JSON value to kickstart Value
        let value = match json_val {
            serde_json::Value::String(s) => Value::String(s.clone()),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Integer(i)
                } else {
                    bail!("Invalid value for '{}': number {} is not a valid integer", key, n);
                }
            }
            serde_json::Value::Bool(b) => Value::Boolean(*b),
            _ => bail!(
                "Invalid value for '{}': only strings, integers and booleans are supported",
                key
            ),
        };

        // Type check
        if var.default.type_str() != value.type_str() {
            bail!(
                "Variable '{}' expects type '{}', but got '{}'",
                key,
                var.default.type_str(),
                value.type_str()
            );
        }

        // Choices validation
        if let Some(ref choices) = var.choices {
            if !choices.contains(&value) {
                let choices_str: Vec<_> = choices.iter().map(|c| format!("{}", c)).collect();
                bail!(
                    "Variable '{}' value '{}' is not in allowed choice: [{}]",
                    key,
                    value,
                    choices_str.join(", ")
                );
            }
        }

        // Regex validation
        if let Some(ref pattern) = var.validation {
            if let Value::String(s) = &value {
                let re = Regex::new(pattern)
                    .with_context(|| format!("Invalid validation regex for '{}'", key))?;
                if !re.is_match(s) {
                    bail!("Variable '{}' value '{}' needs to pass the regex: {}", key, s, pattern);
                }
            }
        }

        values.insert(key.clone(), value);
    }

    Ok(values)
}
