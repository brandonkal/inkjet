// Copyright 2020 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

//! Make your markdown executable with inkjet, the interactive CLI task runner
use colored::*;
use std::env;

fn main() {
    let color = env::var_os("NO_COLOR").is_none();
    let args = env::args().collect();
    let (rc, err_str, prefix) = inkjet::runner::run(args, color);
    if !err_str.is_empty() {
        if prefix {
            eprintln!("{} {}", "ERROR (inkjet):".red(), err_str);
        } else if rc == 0 {
            println!("{}", err_str);
        } else {
            eprintln!("{}", err_str);
        }
    }
    std::process::exit(rc);
}
