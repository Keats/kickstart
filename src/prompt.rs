use std::io::{self, BufRead, Write};

use toml;

use crate::errors::{new_error, ErrorKind, Result};
use crate::interpret;
use crate::terminal;

/// Wait for user input and return what they typed
fn read_line() -> Result<String> {
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let mut lines = stdin.lines();
    lines.next().and_then(|l| l.ok()).ok_or_else(|| new_error(ErrorKind::UnreadableStdin))
}

/// Ask a yes/no question to the user
pub fn ask_bool(prompt: &str, default: bool) -> Result<bool> {
    terminal::bool_question(prompt, default);
    let _ = io::stdout().flush();
    let input = read_line()?;

    let res = match interpret::interpret_bool(&input, default) {
        Ok(x) => x,
        Err(e) => {
            terminal::error(&format!("{}", e));
            ask_bool(prompt, default)?
        }
    };

    Ok(res)
}

/// Ask a question to the user where they can write any string
pub fn ask_string(prompt: &str, default: &str, validation: &Option<String>) -> Result<String> {
    terminal::basic_question(prompt, &default, validation);
    let _ = io::stdout().flush();
    let input = read_line()?;

    let res = match interpret::interpret_string(&input, default, validation) {
        Ok(x) => x,
        Err(e) => {
            terminal::error(&format!("{}", e));
            ask_string(prompt, default, validation)?
        }
    };

    Ok(res)
}

/// Ask a question to the user where they can write an integer
pub fn ask_integer(prompt: &str, default: i64) -> Result<i64> {
    terminal::basic_question(prompt, &default, &None);
    let _ = io::stdout().flush();
    let input = read_line()?;

    let res = match interpret::interpret_integer(&input, default) {
        Ok(x) => x,
        Err(e) => {
            terminal::error(&format!("{}", e));
            ask_integer(prompt, default)?
        }
    };

    Ok(res)
}

/// Ask users to make a choice between various options
pub fn ask_choices(
    prompt: &str,
    default: &toml::Value,
    choices: &[toml::Value],
) -> Result<toml::Value> {
    terminal::bold(&format!("{}: \n", prompt));
    let mut lines = vec![];
    let mut default_index = 1;

    for (index, choice) in choices.iter().enumerate() {
        terminal::bold(&format!("  {}. {}\n", index + 1, choice.as_str().unwrap()));

        lines.push(format!("{}", index + 1));
        if choice == default {
            default_index = index + 1;
        }
    }

    terminal::basic_question(
        &format!("  > Choose from {}..{}", 1, lines.len()),
        &default_index,
        &None,
    );

    let _ = io::stdout().flush();
    let input = read_line()?;

    let res = match interpret::interpret_choices(&input, default, choices) {
        Ok(x) => x,
        Err(e) => {
            terminal::error(&format!("{}", e));
            ask_choices(prompt, default, choices)?
        }
    };

    Ok(res)
}
