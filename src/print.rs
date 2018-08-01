use std::io::prelude::*;
use std::fmt;

use term;


pub fn error(message: &str) {
    if let Some(mut t) = term::stderr() {
        match t.fg(term::color::BRIGHT_RED) {
            Ok(_) => {
                write!(t, "{}", message).unwrap();
                t.reset().unwrap();
            },
            Err(_) => writeln!(t, "{}", message).unwrap()
        };
    } else {
        eprint!("{}", message);
    }
}


pub fn success(message: &str) {
    if let Some(mut t) = term::stdout() {
        match t.fg(term::color::BRIGHT_GREEN) {
            Ok(_) => {
                write!(t, "{}", message).unwrap();
                t.reset().unwrap();
            },
            Err(_) => writeln!(t, "{}", message).unwrap()
        };
    } else {
        eprint!("{}", message);
    }
}

pub fn bold(message: &str) {
    if let Some(mut t) = term::stdout() {
        match t.attr(term::Attr::Bold) {
            Ok(_) => {
                write!(t, "{}", message).unwrap();
                t.reset().unwrap();
            },
            Err(_) => write!(t, "{}", message).unwrap()
        };
    } else {
        eprint!("{}", message);
    }
}

pub fn basic_question<T: fmt::Display>(prompt: &str, default: &T, validation: &Option<String>) {
    if let Some(mut t) = term::stdout() {
        // check for colour/boldness at the beginning so we can unwrap later
        if !t.supports_color() || !t.supports_attr(term::Attr::Bold) {
            if let Some(ref pattern) = validation {
                write!(t, "{} [default: {}, validation: {}]: ", prompt, default, pattern).unwrap();
            } else {
                write!(t, "{} [default: {}]: ", prompt, default).unwrap();
            }
            return;
        }

        t.attr(term::Attr::Bold).unwrap();
        write!(t, "{} ", prompt).unwrap();
        t.reset().unwrap();
        t.fg(term::color::YELLOW).unwrap();
        if let Some(ref pattern) = validation {
            write!(t, "[default: {}, validation: {}]: ", default, pattern).unwrap();
        } else {
            write!(t, "[default: {}]: ", default).unwrap();
        }
        t.reset().unwrap();
    } else {
        eprint!("{} [default: {}]: ", prompt, default);
    }
}

pub fn bool_question(prompt: &str, default: bool) {
    if let Some(mut t) = term::stdout() {
        // check for colour/boldness at the beginning so we can unwrap later
        if !t.supports_color() || !t.supports_attr(term::Attr::Bold) {
            write!(t, "{} {}: ", prompt, if default { "[Y/n]" } else { "[y/N" }).unwrap();
            return;
        }

        t.attr(term::Attr::Bold).unwrap();
        write!(t, "{} ", prompt).unwrap();
        t.reset().unwrap();
        t.fg(term::color::YELLOW).unwrap();
        if default {  write!(t, "[Y/n]").unwrap() } else { write!(t, "[y/N]").unwrap() }
        t.reset().unwrap();
    } else {
        eprint!("{} [default: {}]: ", prompt, default);
    }
}
