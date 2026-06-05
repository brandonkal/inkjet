// Copyright 2025 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

/// Plain informational message prefix.
pub const PLAIN_INFO_MSG: &str = "INFO (inkjet):";
/// Plain warning message prefix.
pub const PLAIN_WARNING_MSG: &str = "WARNING (inkjet):";
/// Plain error message prefix.
pub const PLAIN_ERROR_MSG: &str = "ERROR (inkjet):";
/// Plain invalid input message prefix.
pub const PLAIN_INVALID_MSG: &str = "INVALID:";

/// Colored informational message prefix.
pub const COLOR_INFO_MSG: &str =
    color_print::cstr!("<underline><yellow>INFO (inkjet):</yellow></underline>");
/// Colored warning message prefix.
pub const COLOR_WARNING_MSG: &str =
    color_print::cstr!("<underline><yellow>WARNING (inkjet):</yellow></underline>");
/// Colored error message prefix.
pub const COLOR_ERROR_MSG: &str =
    color_print::cstr!("<underline><red>ERROR (inkjet):</red></underline>");
/// Colored invalid input message prefix.
pub const COLOR_INVALID_MSG: &str =
    color_print::cstr!("<underline><red>INVALID:</red></underline>");

/// Returns the informational message prefix for the color setting.
pub fn info_msg(color: bool) -> &'static str {
    if color {
        COLOR_INFO_MSG
    } else {
        PLAIN_INFO_MSG
    }
}

/// Returns the warning message prefix for the color setting.
pub fn warning_msg(color: bool) -> &'static str {
    if color {
        COLOR_WARNING_MSG
    } else {
        PLAIN_WARNING_MSG
    }
}

/// Returns the error message prefix for the color setting.
pub fn error_msg(color: bool) -> &'static str {
    if color {
        COLOR_ERROR_MSG
    } else {
        PLAIN_ERROR_MSG
    }
}

/// Returns the invalid input message prefix for the color setting.
pub fn invalid_msg(color: bool) -> &'static str {
    if color {
        COLOR_INVALID_MSG
    } else {
        PLAIN_INVALID_MSG
    }
}

/// Returns the informational message prefix based on the NO_COLOR environment variable.
pub fn info_msg_from_env() -> &'static str {
    info_msg(std::env::var_os("NO_COLOR").is_none())
}

/// Returns the error message prefix based on the NO_COLOR environment variable.
pub fn error_msg_from_env() -> &'static str {
    error_msg(std::env::var_os("NO_COLOR").is_none())
}
