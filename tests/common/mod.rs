#![warn(clippy::indexing_slicing)]
use assert_cmd::{crate_name, prelude::*};
use assert_fs::prelude::*;
use std::path::PathBuf;
use std::process::Command;

pub trait InkjetCommandExt {
    fn command(&mut self, c: &'static str) -> &mut Command;
    fn cli(&mut self, arguments: &'static str) -> &mut Command;
}

impl InkjetCommandExt for Command {
    fn command(&mut self, c: &'static str) -> &mut Command {
        self.arg(c);
        self
    }

    fn cli(&mut self, arguments: &'static str) -> &mut Command {
        let args: Vec<&str> = arguments.split_whitespace().collect();
        for arg in args {
            self.arg(arg);
        }
        self
    }
}

pub fn inkfile(content: &'static str) -> (assert_fs::TempDir, PathBuf) {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let inkfile = temp_dir.child("inkjet.md");
    inkfile.write_str(content).unwrap();
    let inkfile_path = inkfile.path().to_path_buf();
    (temp_dir, inkfile_path)
}

pub fn run_binary() -> Command {
    Command::cargo_bin(crate_name!()).expect("Was not able to find binary")
}

pub fn run_inkjet(inkfile: &PathBuf) -> Command {
    let mut inkjet = run_binary();
    inkjet.arg("--inkfile").arg(inkfile);
    inkjet
}
