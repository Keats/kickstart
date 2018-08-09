//! # Kickstart
//! `kickstart` is a CLI application that allows a user to quickly
//! get started on a project based on a template
//! It is also available as a library in case you want to integrate it in your own CLI
//! application.
//! See the [kickstart binary](https://github.com/Keats/kickstart/blob/master/src/bin/kickstart.rs)
//! for an example on how to use it.

extern crate tera;
extern crate walkdir;
extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate memchr;
extern crate glob;
extern crate regex;
extern crate term;
#[cfg(test)]
extern crate tempfile;

mod definition;
mod prompt;
mod utils;
pub mod terminal;
pub mod validate;
pub mod generation;
pub mod errors;
