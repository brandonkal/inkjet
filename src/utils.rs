// Copyright 2025 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

/// returns INFO string (yellow if NO_COLOR is unset).
pub const INFO_MSG: &str =
    color_print::cstr!("<underline><yellow>INFO (inkjet):</yellow></underline>");

/// returns WARNING string (yellow if NO_COLOR is unset).
pub const WARNING_MSG: &str =
    color_print::cstr!("<underline><yellow>WARNING (inkjet):</yellow></underline>");

/// returns ERROR string (red if NO_COLOR is unset).
pub const ERROR_MSG: &str = color_print::cstr!("<underline><red>ERROR (inkjet):</red></underline>");

/// returns INVALID string (red if NO_COLOR is unset).
pub const INVALID_MSG: &str = color_print::cstr!("<underline><red>INVALID:</red></underline>");
