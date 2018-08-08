use std::fs::{File, create_dir_all};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use walkdir::DirEntry;
use memchr::memchr;

use errors::{Result, ErrorKind, new_error};


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

pub fn is_vcs(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with(".git"))
        .unwrap_or(false)
}

/// See https://twitter.com/20100Prouillet/status/1022973478096527360
pub fn is_binary(buf: &[u8]) -> bool {
    memchr(b'\x00', buf).is_some()
}

#[cfg(test)]
mod tests {
    // TODO tests, I don't really know how to test those here
    // use super::*;

    // #[test]
    // fn can_detect_git_source() {
    //     let ssh = "gitUser@git-server.local:git/Test";
    //     let http = "https://git-server.local/git/Test";
    //     let shortened = "git:git/Test";
    //     let invalid = "test";

    //     match get_source(ssh) {
    //         Source::Git(ref _res) => {},
    //         Source::Local(ref _res) => panic!("Expected {} to be considered as Git, got Local", ssh),
    //     }
    //     match get_source(http) {
    //         Source::Git(ref _res) => {},
    //         Source::Local(ref _res) => panic!("Expected {} to be considered as Git, got Local", http),
    //     }
    //     match get_source(shortened) {
    //         Source::Git(ref _res) => {},
    //         Source::Local(ref _res) => panic!("Expected {} to be considered as Git, got Local", shortened),
    //     }
    //     match get_source(invalid) {
    //         Source::Git(ref _res) => panic!("Expected {} to be considered as Local, got Git", invalid),
    //         Source::Local(ref _res) => {},
    //     }
    // }

    // #[test]
    // fn can_detect_local_sources() {
    //     let invalid = "git:git/Test";
    //     let relative_valid = "test/abc/def";
    //     let absolute_valid = "test/abc/def";

    //     match get_source(relative_valid) {
    //         Source::Git(ref _res) => panic!("Expected {} to be considered as Local, got Git", relative_valid),
    //         Source::Local(ref _res) => {},
    //     }
    //     match get_source(absolute_valid) {
    //         Source::Git(ref _res) => panic!("Expected {} to be considered as Local, got Git", absolute_valid),
    //         Source::Local(ref _res) => {},
    //     }
    //     match get_source(invalid) {
    //         Source::Git(ref _res) => {},
    //         Source::Local(ref _res) => panic!("Expected {} to be considered as Git, got Local", invalid),
    //     }
    // }
}
