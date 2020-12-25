use regex::Regex;

use crate::errors::{new_error, ErrorKind, Result};

/// Interpret a yes/no question to the user
pub fn interpret_bool(input: &str, default: bool) -> Result<bool> {
    match &*input {
        "y" | "Y" | "yes" | "YES" | "true" => Ok(true),
        "n" | "N" | "no" | "NO" | "false" => Ok(false),
        "" => Ok(default),
        _ => Err(new_error(ErrorKind::InvalidInput {
            msg: format!("Invalid choice: '{}'\n", input),
        })),
    }
}

/// Interpret a question to the user where they can write any string
pub fn interpret_string(input: &str, default: &str, validation: &Option<String>) -> Result<String> {
    match &*input {
        "" => Ok(default.to_string()),
        _ => {
            if let Some(ref pattern) = validation {
                let re = Regex::new(pattern).unwrap();
                if re.is_match(&input) {
                    Ok(String::from(input))
                } else {
                    Err(new_error(ErrorKind::InvalidInput {
                        msg: format!("The value needs to pass the regex: {}\n", pattern),
                    }))
                }
            } else {
                Ok(String::from(input))
            }
        }
    }
}

/// Interpret a question to the user where they can write an integer
pub fn interpret_integer(input: &str, default: i64) -> Result<i64> {
    match &*input {
        "" => Ok(default),
        _ => match input.parse::<i64>() {
            Ok(i) => Ok(i),
            Err(_) => Err(new_error(ErrorKind::InvalidInput {
                msg: format!("Invalid integer: '{}'\n", input),
            })),
        },
    }
}

/// Interpret a choice between various options
pub fn interpret_choices(
    input: &str,
    default: &toml::Value,
    choices: &[toml::Value],
) -> Result<toml::Value> {
    match &*input {
        "" => Ok(default.clone()),
        _ => {
            if let Ok(num) = input.parse::<usize>() {
                if num > choices.len() {
                    Err(new_error(ErrorKind::InvalidInput {
                        msg: format!("Invalid choice: '{}'\n", input),
                    }))
                } else {
                    Ok(choices[num - 1].clone())
                }
            } else {
                Err(new_error(ErrorKind::InvalidInput {
                    msg: format!("Invalid choice: '{}'\n", input),
                }))
            }
        }
    }
}
