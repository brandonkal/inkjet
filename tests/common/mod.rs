#![warn(clippy::indexing_slicing)]
// Copyright 2020 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

use assert_cmd::{cargo, crate_name, prelude::*};
use assert_fs::prelude::*;
use std::env;
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

/// Returns the path for the current binary under this integration test
pub fn cargo_bin() -> String {
    let path = cargo::cargo_bin(crate_name!());
    if path.is_file() {
        return path.to_string_lossy().to_string();
    }
    panic!("Could not locate cargo_bin {:?}", path)
}

/// Returns temp directory to support Windows testing
pub fn temp_path() -> String {
    #[cfg(not(windows))]
    let temp_dir = "/tmp";
    #[cfg(windows)]
    let temp_dir = env::var("TEMP").expect("Test error: Could not read %TEMP%");

    temp_dir.to_string()
}
