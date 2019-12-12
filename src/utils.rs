use std::fs::{File, create_dir_all};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use walkdir::DirEntry;
use memchr::memchr;

use crate::errors::{Result, ErrorKind, new_error};


#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Source {
    Local(PathBuf),
    Git(String),
}

pub fn read_file(p: &Path) -> Result<String> {
    let mut f = match File::open(p) {
        Ok(f) => f,
        Err(err) => return Err(new_error(ErrorKind::Io { err, path: p.to_path_buf() }))
    };


    let mut contents = String::new();
    match f.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(err) => return Err(new_error(ErrorKind::Io { err, path: p.to_path_buf() }))
    };

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
    let path = Path::new(input);

    if path.is_dir() {
        Source::Local(path.to_path_buf())
    } else {
        Source::Git(input.to_string())
    }
}

/// Is this entry from a VCS?
pub fn is_vcs(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with(".git"))
        .unwrap_or(false)
}

/// Is the buffer from a binary file?
/// See https://twitter.com/20100Prouillet/status/1022973478096527360
pub fn is_binary(buf: &[u8]) -> bool {
    memchr(b'\x00', buf).is_some()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::tempdir;
    use super::*;

    #[test]
    fn can_detect_sources() {
        let dir = tempdir().unwrap();
        let folder1 = dir.path().join("working");
        let folder2 = dir.path().join("also-working");
        fs::create_dir(&folder1).unwrap();
        fs::create_dir(&folder2).unwrap();
        let mut inputs = vec![
            // Local valid
            (folder1.to_string_lossy().to_string(), Source::Local(folder1.to_path_buf())),
            (folder2.to_string_lossy().to_string(), Source::Local(folder2.to_path_buf())),
            // Git valid
            ("https://git-server.local/git/Test".to_string(), Source::Git("https://git-server.local/git/Test".to_string())),
            ("gitUser@git-server.local:git/Test".to_string(), Source::Git("gitUser@git-server.local:git/Test".to_string())),
            ("git:git/Test".to_string(), Source::Git("git:git/Test".to_string())),
            // Non existing local -> considered as a git and will fail later on
            ("hello".to_string(), Source::Git("hello".to_string())),
        ];
        if !cfg!(windows) {
            let folder3 = dir.path().join("not:git");
            fs::create_dir(&folder3).unwrap();
            inputs.push((folder3.to_string_lossy().to_string(), Source::Local(folder3.to_path_buf())));
        }
        for (input, expected) in inputs {
            assert_eq!(get_source(&input), expected);
        }
    }
}
