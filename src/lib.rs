//! # Kickstart
//! `kickstart` is a CLI application that allows a user to quickly
//! get started on a project based on a template
//! It is also available as a library in case you want to integrate it in your own CLI
//! application.
//! See the [kickstart binary](https://github.com/Keats/kickstart/blob/master/src/bin/kickstart.rs)
//! for an example on how to use it.

mod definition;
pub mod errors;
mod filters;
mod generation;
pub mod prompt;
pub mod terminal;
pub mod utils;
mod value;

pub use definition::{Cleanup, Condition, Hook, TemplateDefinition, Variable};
pub use generation::{HookFile, Template};
pub use value::Value;
