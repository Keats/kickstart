use std::io::{self, Write, BufRead};


/// Wait for user input and return what they typed
fn read_line() -> String {
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let mut lines = stdin.lines();
    lines
        .next()
        .and_then(|l| l.ok())
        .unwrap()
}

/// Ask a yes/no question to the user
pub fn ask_bool(question: &str, default: bool) -> bool {
    print!("{} {}: ", question, if default { "[Y/n]" } else { "[y/N]" });
    let _ = io::stdout().flush();
    let input = read_line();

    match &*input {
        "y" | "Y" | "yes" | "YES" | "true" => true,
        "n" | "N" | "no" | "NO" | "false" => false,
        "" => default,
        _ => {
            println!("Invalid choice: '{}'", input);
            ask_bool(question, default)
        },
    }
}

/// Ask a question to the user where they can write any string
pub fn ask_string(question: &str, default: &str) -> String {
    print!("{} ({}): ", question, default);
    let _ = io::stdout().flush();
    let input = read_line();

    match &*input {
        "" => default.to_string(),
        _ => input,
    }
}
