use std::io::{self, Write, BufRead};

use regex::Regex;
use toml;

use errors::{Result, new_error, ErrorKind};
use print;

/// Wait for user input and return what they typed
fn read_line() -> Result<String> {
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let mut lines = stdin.lines();
    lines
        .next()
        .and_then(|l| l.ok())
        .ok_or_else(|| new_error(ErrorKind::UnreadableStdin))
}

/// Ask a yes/no question to the user
pub fn ask_bool(prompt: &str, default: bool) -> Result<bool> {
    print::bold(&format!("- {} {}: ", prompt, if default { "[Y/n]" } else { "[y/N]" }));
    let _ = io::stdout().flush();
    let input = read_line()?;

    let res = match &*input {
        "y" | "Y" | "yes" | "YES" | "true" => true,
        "n" | "N" | "no" | "NO" | "false" => false,
        "" => default,
        _ => {
            print::error(&format!("Invalid choice: '{}'\n", input));
            ask_bool(prompt, default)?
        },
    };

    Ok(res)
}

/// Ask a question to the user where they can write any string
pub fn ask_string(prompt: &str, default: &str, validation: &Option<String>) -> Result<String> {
    if let Some(ref pattern) = validation {
        print::bold(&format!("- {} [must match {}] ({}): ", prompt, pattern, default));
    } else {
        print::bold(&format!("- {} ({}): ", prompt, default));
    }
    let _ = io::stdout().flush();
    let input = read_line()?;

    let res = match &*input {
        "" => default.to_string(),
        _ => {
            if let Some(ref pattern) = validation {
                let re = Regex::new(pattern).unwrap();
                if re.is_match(&input) {
                    input
                } else {
                    print::error(&format!("The value needs to pass the regex: {}\n", pattern));
                    ask_string(prompt, default, validation)?
                }
            } else {
                input
            }
        },
    };

    Ok(res)
}

/// Ask a question to the user where they can write an integer
pub fn ask_integer(prompt: &str, default: i64) -> Result<i64> {
    print::bold(&format!("- {} ({}): ", prompt, default));
    let _ = io::stdout().flush();
    let input = read_line()?;

    let res = match &*input {
        "" => default,
        _ => match input.parse::<i64>() {
            Ok(i) => i,
            Err(_) => {
                print::error(&format!("Invalid integer: '{}'\n", input));
                ask_integer(prompt, default)?
            }
        },
    };

    Ok(res)
}

/// Ask users to make a choice between various options
pub fn ask_choices(prompt: &str, default: &toml::Value, choices: &toml::value::Array) -> Result<toml::Value> {
    print::bold(&format!("- {}: ", prompt));
    let mut lines = vec![];
    let mut default_index = 1;

    for (index, choice) in choices.iter().enumerate() {
        print::bold(&format!("\n{}. {}", index + 1, choice.as_str().unwrap()));

        lines.push(format!("{}", index + 1));
        if choice == default {
            default_index = index + 1;
        }
    }

    print::bold(&format!("\n> Choose from {} ({}): ", lines.join(", "), default_index));

    let _ = io::stdout().flush();
    let input = read_line()?;

    let res = match &*input {
        "" => default.clone(),
        _ => {
            if let Ok(num) = input.parse::<usize>() {
                if num > choices.len() {
                    print::error(&format!("Invalid choice: '{}'\n", input));
                    ask_choices(prompt, default, choices)?
                } else {
                    choices.get(num - 1).unwrap().clone()
                }
            } else {
                print::error(&format!("Invalid choice: '{}'\n", input));
                ask_choices(prompt, default, choices)?
            }
        },
    };

    Ok(res)
}
