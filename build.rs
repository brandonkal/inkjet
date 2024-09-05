// Copyright 2020 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

#[cfg(windows)]
fn main() {
    let mut res = tauri_winres::WindowsResource::new();
    res.set_icon("inkjet-icon.ico");
    res.compile().unwrap();
}

#[cfg(unix)]
fn main() {}