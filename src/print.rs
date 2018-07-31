use std::io;
use std::io::prelude::*;

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
        write!(io::stderr(), "{}", message).unwrap();
    }
}


pub fn success(message: &str) {
    if let Some(mut t) = term::stdout() {
        match t.fg(term::color::GREEN) {
            Ok(_) => {
                write!(t, "{}", message).unwrap();
                t.reset().unwrap();
            },
            Err(_) => writeln!(t, "{}", message).unwrap()
        };
    } else {
        write!(io::stderr(), "{}", message).unwrap();
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
        write!(io::stderr(), "{}", message).unwrap();
    }
}
