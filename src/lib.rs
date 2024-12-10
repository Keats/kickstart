//! # Kickstart
//! `kickstart` is a framework to generate projects based on templates.
//! It is available as a library in case you want to use it in your own program and as a CLI.
//! See the [kickstart binary](https://github.com/Keats/kickstart/blob/master/src/main.rs)
//! for an example on how to use the library.

#[cfg(feature = "cli")]
pub mod cli;
mod definition;
pub mod errors;
mod filters;
mod generation;
mod utils;
mod value;

pub use definition::{Cleanup, Condition, Hook, TemplateDefinition, Variable};
pub use generation::{HookFile, Template};
pub use value::Value;
