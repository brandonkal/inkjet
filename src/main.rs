// Copyright 2020 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

//! Make your markdown executable with inkjet, the interactive CLI task runner
use std::env;

fn main() {
    let color = env::var_os("NO_COLOR").is_none();
    let args = env::args().collect();
    let rc = inkjet::runner::run(args, color);
    std::process::exit(rc);
}
