use std::fs::{File, create_dir_all};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use toml::{self, Value as TomlValue};
use tera::{Tera, Context};
use walkdir::WalkDir;

use prompt::{ask_string, ask_bool};


// TODO: error handling
pub fn read_file(p: &Path) -> String {
    let mut f = File::open(p).expect("file not found");

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    contents
}

pub fn write_file(p: &Path, contents: &str) {
    let mut f = File::create(p).expect("Unable to create file");
    f.write_all(contents.as_bytes()).expect("Unable to write data");
}

pub fn create_directory(path: &Path) {
    if !path.exists() {
        create_dir_all(path).unwrap();
    }
}

#[derive(Debug, PartialEq)]
pub struct Template {
    path: PathBuf,
    name: String,
}

impl Template {
    pub fn from_cli(path: &str) -> Template {
        // TODO: templates can be from git as well, not only local
        let path = Path::new(path);
        let name = path.file_name().unwrap().to_string_lossy();

        Template {
            path: path.to_path_buf(),
            name: name.to_string(),
        }
    }

    pub fn download_from_vcs(&self) {
        // TODO
    }

    fn ask_questions(&self, conf: TomlValue) -> Context {
        let table = conf.as_table().unwrap();
        let mut context = Context::new();

        for (key, data) in table {
            let question = data["question"].as_str().unwrap();
            if let Some(b) = data["default"].as_bool() {
                let res = ask_bool(question, b);
                context.add(key, &res);

            } else {
                let res = ask_string(question, data["default"].as_str().unwrap());
                context.add(key, &res);
            };
        }

        context
    }

    // TODO: error handling
    pub fn generate(&self, output_dir: PathBuf) {
        // Get the variables from the user first
        let conf_path = self.path.join("template.toml");
        if !conf_path.exists() {
            panic!("template.toml missing")
        }
        let conf: TomlValue = toml::from_str(&read_file(&conf_path)).unwrap();
        let context = self.ask_questions(conf);

        if !output_dir.exists() {
            create_directory(&output_dir);
        }

        // And now generate the files (in current directory for now)
        for entry in WalkDir::new(&self.path).into_iter().filter_map(|e| e.ok()) {
            // Skip root folder and the template.toml
            if entry.path() == self.path || entry.path() == conf_path {
                continue;
            }
            let path = entry.path().strip_prefix(&self.path).unwrap();
            // TODO: only render tpl if {{..}}
            let tpl = Tera::one_off(&format!("{}", path.display()), &context, false).unwrap();
            let real_path = Path::new(&tpl);

            if entry.path().is_dir() {
                create_directory(&output_dir.join(real_path));
            } else {
                let contents = Tera::one_off(&read_file(&entry.path()), &context, false).unwrap();
                write_file(&output_dir.join(real_path), &contents);
            }
        }
    }
}
