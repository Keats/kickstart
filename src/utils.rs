use std::fs::{File, create_dir_all};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::DirEntry;

use errors::{Result, ResultExt};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Source {
    Local(PathBuf),
    Git(String),
}

pub fn read_file(p: &Path) -> Result<String> {
    let mut f = File::open(p)
        .chain_err(|| format!("File {} not found or not readable", p.display()))?;

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .chain_err(|| format!("Could not read {}", p.display()))?;

    Ok(contents)
}

pub fn write_file(p: &Path, contents: &str) -> Result<()> {
    let mut f = File::create(p)?;
    f.write_all(contents.as_bytes())?;
    Ok(())
}

pub fn create_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        create_dir_all(path)?;
    }

    Ok(())
}

/// Is it a remote or a local thing
pub fn get_source(input: &str) -> Source {
    // Should be a Regex once we add hg or other stuff
    if input.starts_with("git@") || input.starts_with("http://") || input.starts_with("https://") {
        Source::Git(input.to_string())
    } else {
        Source::Local(Path::new(input).to_path_buf())
    }
}

pub fn is_vcs(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with(".git"))
         .unwrap_or(false)
}
