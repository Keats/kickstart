use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::result;

/// A crate private constructor for `Error`.
pub(crate) fn new_error(kind: ErrorKind) -> Error {
    Error { kind, source: None }
}

/// Map an IO error, providing a path for context.
pub(crate) fn map_io_err<T>(err: io::Result<T>, path: &Path) -> Result<T> {
    err.map_err(|err| new_error(ErrorKind::Io { err, path: path.to_path_buf() }))
}

/// A type alias for `Result<T, kickstart::Error>`.
pub type Result<T> = result::Result<T, Error>;

/// The Error type
#[derive(Debug)]
pub struct Error {
    /// Kind of error
    pub kind: ErrorKind,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

/// The specific type of an error.
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    MissingTemplateDefinition,
    InvalidTemplate,
    UnreadableStdin,
    InvalidVariableName(String),
    /// An error while cloning a repository
    Git {
        err: io::Error,
    },
    /// An error while doing IO (reading/writing files)
    Io {
        err: io::Error,
        path: PathBuf,
    },
    /// An error when rendering a template, where a template can also refer to a filename
    /// in the case kickstart
    Tera {
        err: tera::Error,
        path: Option<PathBuf>,
    },
    /// An error while deserializing a template.toml into a struct
    Toml {
        err: toml::de::Error,
    },
    /// A glob pattern couldn't be built from the input
    InvalidGlobPattern {
        pattern_before_rendering: String,
        pattern_after_rendering: Option<String>,
        err: glob::PatternError,
    },
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        new_error(ErrorKind::Io { err, path: PathBuf::new() })
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as _)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Io { ref err, ref path } => write!(f, "{}: {:?}", err, path),
            ErrorKind::Tera { ref err, ref path } => {
                if let Some(p) = path {
                    write!(f, "{}: {:?}", err, p)
                } else {
                    write!(f, "{}: rendering a one-off template", err)
                }
            }
            ErrorKind::Git { ref err } => write!(f, "Could not clone the repository: {}", err),
            ErrorKind::Toml { ref err } => write!(f, "Invalid TOML: {}", err),
            ErrorKind::InvalidGlobPattern {
                ref err,
                ref pattern_before_rendering,
                ref pattern_after_rendering,
            } => {
                if let Some(after) = pattern_after_rendering {
                    write!(
                        f,
                        "Invalid glob pattern `{}` (before rendering: {}): {}",
                        after, pattern_before_rendering, err
                    )
                } else {
                    write!(f, "Invalid glob pattern `{}`: {}", pattern_before_rendering, err)
                }
            }
            ErrorKind::InvalidVariableName(ref name) => {
                write!(f, "Variable {name} not found in the template definition")
            }
            ErrorKind::MissingTemplateDefinition => write!(f, "The template.toml is missing"),
            ErrorKind::UnreadableStdin => write!(f, "Unable to read from stdin"),
            ErrorKind::InvalidTemplate => write!(f, "The template.toml is invalid"),
        }
    }
}
