// Copyright 2020 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

use mdcat::{ResourceAccess, TerminalCapabilities, TerminalSize};
use pulldown_cmark::{Options, Parser};
use std::error::Error;
use std::io::stderr;
use std::path::Path;
use syntect::parsing::SyntaxSet;

/// The Printer represents an instance for printing markdown to the terminal.
pub struct Printer {
    size: TerminalSize,
    resource_access: ResourceAccess,
    terminal_capabilities: TerminalCapabilities,
    syntax_set: SyntaxSet,
    base_dir: String,
}

impl Printer {
    #[must_use]
    /// Build a new Printer for printing markdown to the terminal.
    pub fn new(colors: bool, local_only: bool, filename: &str) -> Printer {
        let terminal_capabilities = if !colors {
            // If the user disabled colours assume a dumb terminal
            TerminalCapabilities::none()
        } else {
            TerminalCapabilities::detect()
        };
        let resource_access = if local_only {
            ResourceAccess::LocalOnly // available for library users
        } else {
            ResourceAccess::RemoteAllowed
        };
        let syntax_set = SyntaxSet::load_defaults_newlines();

        // On Windows 10 we need to enable ANSI term explicitly.
        #[cfg(windows)]
        {
            ansi_term::enable_ansi_support().ok();
        }

        Printer {
            size: TerminalSize::detect().unwrap_or_default(),
            terminal_capabilities,
            resource_access,
            syntax_set,
            base_dir: Path::new(&filename)
                .parent()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        }
    }

    /// Parses a given markdown string and renders it to the terminal.
    pub fn print_markdown(&self, input: &str) -> Result<(), Box<dyn Error>> {
        mdcat::push_tty(
            &mut stderr(),
            &self.terminal_capabilities,
            TerminalSize {
                // width: self.size.width.to_string(),
                ..self.size
            },
            create_markdown_parser(input),
            Path::new(&self.base_dir),
            self.resource_access,
            self.syntax_set.clone(),
        )
    }
}

fn create_markdown_parser(contents: &str) -> Parser {
    // Set up options and parser. Strikethroughs are not part of the CommonMark standard
    // and we therefore must enable it explicitly.
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    Parser::new_ext(contents, options)
}

#[test]
fn make_printer() {
    let p = Printer::new(false, true, "folder/somefile.txt");
    assert_eq!(p.base_dir, "folder".to_string());
}
