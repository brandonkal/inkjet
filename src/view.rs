// Copyright 2020 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

use pulldown_cmark::{Options, Parser};
use pulldown_cmark_mdcat::resources::FileResourceHandler;
use pulldown_cmark_mdcat::terminal::TerminalSize;
use pulldown_cmark_mdcat::{Environment, Settings, TerminalProgram, Theme, push_tty};
use std::error::Error;
use std::io::stderr;
use std::path::Path;
use syntect::parsing::SyntaxSet;

/// The Printer represents an instance for printing markdown to the terminal.
pub struct Printer {
    syntax_set: SyntaxSet,
    terminal_program: TerminalProgram,
    environment: Environment,
}

impl Printer {
    #[must_use]
    /// Build a new Printer for printing markdown to the terminal.
    pub fn new(colors: bool, filename: &str) -> Printer {
        let syntax_set = SyntaxSet::load_defaults_newlines();

        // Determine terminal capabilities based on colors setting
        let terminal_program: TerminalProgram = if !colors {
            TerminalProgram::Ansi
        } else {
            TerminalProgram::detect()
        };

        let base_dir = Path::new(&filename)
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        // Create environment
        let environment = Environment::for_local_directory(&base_dir).unwrap_or(Environment {
            base_url: url::Url::parse("http://localhost/").expect("Failed to parse URL"),
            hostname: String::from("localhost"),
        });

        Printer {
            syntax_set,
            terminal_program,
            environment,
        }
    }

    /// Parses a given markdown string and renders it to the terminal.
    pub fn print_markdown(&self, input: &str) -> Result<(), Box<dyn Error>> {
        // Create a resource handler
        let resource_handler = FileResourceHandler::new(u64::MAX);

        // Load a theme
        let theme = Theme::default();
        let terminal_capabilities = TerminalProgram::capabilities(self.terminal_program);

        // Create settings
        let settings = Settings {
            terminal_capabilities,
            terminal_size: TerminalSize::detect().unwrap_or_default(),
            theme,
            syntax_set: &self.syntax_set,
        };

        // Create parser
        let parser = create_markdown_parser(input);

        // Convert the result to Box<dyn Error>
        match push_tty(
            &settings,
            &self.environment,
            &resource_handler,
            &mut stderr(),
            parser,
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
}

fn create_markdown_parser(contents: &'_ str) -> Parser<'_> {
    // Set up options and parser. Strikethroughs are not part of the CommonMark standard
    // and we therefore must enable it explicitly.
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    Parser::new_ext(contents, options)
}

#[test]
fn make_printer() {
    let p = Printer::new(false, "folder/somefile.txt");
    assert_eq!(
        p.environment.base_url.to_string(),
        "http://localhost/".to_string()
    );
}
