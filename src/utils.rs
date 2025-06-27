// Copyright 2025 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

use colored::{ColoredString, Colorize};
use std::env;

// colored prints the message as a yellow or red string only if NO_COLOR is unset.
fn colored(message: &str, is_red: bool) -> ColoredString {
    // Check if NO_COLOR is set
    let use_color = env::var_os("NO_COLOR").is_none();

    if use_color {
        if is_red {
            message.red() // Return the message colored red
        } else {
            message.yellow() // Return the message colored yellow
        }
    } else {
        message.to_string().into() // Return uncolored message if NO_COLOR is set
    }
}

/// returns INFO string (yellow if NO_COLOR is unset).
pub fn info_msg() -> ColoredString {
    colored("INFO (inkjet):", false)
}

/// returns WARNING string (yellow if NO_COLOR is unset).
pub fn warn_msg() -> ColoredString {
    colored("WARNING (inkjet):", false)
}

/// returns ERROR string (red if NO_COLOR is unset).
pub fn error_msg() -> ColoredString {
    colored("ERROR (inkjet):", true)
}

/// returns INVALID string (red if NO_COLOR is unset).
pub fn invalid_msg() -> ColoredString {
    colored("INVALID:", true)
}
