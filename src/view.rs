// Copyright 2020 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use std::error::Error;
use std::io::{self, Write, stderr};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{LinesWithEndings, as_24_bit_terminal_escaped};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const ITALIC: &str = "\x1b[3m";
const UNDERLINE: &str = "\x1b[4m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";

/// The Printer represents an instance for printing markdown to the terminal.
pub struct Printer {
    syntax_set: SyntaxSet,
    theme: Theme,
    colors: bool,
}

impl Printer {
    #[must_use]
    /// Build a new Printer for printing markdown to the terminal.
    pub fn new(colors: bool, _filename: &str) -> Printer {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme = ThemeSet::load_defaults()
            .themes
            .remove("base16-ocean.dark")
            .unwrap_or_default();

        Printer {
            syntax_set,
            theme,
            colors,
        }
    }

    /// Parses a given markdown string and renders it to the terminal.
    pub fn print_markdown(&self, input: &str) -> Result<(), Box<dyn Error>> {
        let parser = create_markdown_parser(input);
        let mut renderer =
            TerminalMarkdownRenderer::new(self.colors, &self.syntax_set, &self.theme);
        let mut buffer = Vec::new();
        renderer.render(parser, &mut buffer)?;

        while matches!(buffer.last(), Some(b'\n' | b'\r')) {
            buffer.pop();
        }
        buffer.push(b'\n');
        buffer.push(b'\n');

        stderr().write_all(&buffer)?;
        Ok(())
    }
}

struct TerminalMarkdownRenderer<'a> {
    colors: bool,
    syntax_set: &'a SyntaxSet,
    theme: &'a Theme,
    text: String,
    list_stack: Vec<Option<u64>>,
    quote_depth: usize,
    code_lang: Option<String>,
    code: String,
    in_code: bool,
    in_heading: bool,
    in_link: bool,
    link_dest: String,
    paragraph_open: bool,
}

impl<'a> TerminalMarkdownRenderer<'a> {
    fn new(colors: bool, syntax_set: &'a SyntaxSet, theme: &'a Theme) -> Self {
        Self {
            colors,
            syntax_set,
            theme,
            text: String::new(),
            list_stack: Vec::new(),
            quote_depth: 0,
            code_lang: None,
            code: String::new(),
            in_code: false,
            in_heading: false,
            in_link: false,
            link_dest: String::new(),
            paragraph_open: false,
        }
    }

    fn render<W: Write>(&mut self, parser: Parser<'_>, out: &mut W) -> io::Result<()> {
        for event in parser {
            match event {
                Event::Start(tag) => self.start(tag, out)?,
                Event::End(tag) => self.end(tag, out)?,
                Event::Text(text) => self.write_text(&text, out)?,
                Event::Code(text) => self.write_inline_code(&text, out)?,
                Event::Html(html) | Event::InlineHtml(html) => self.write_text(&html, out)?,
                Event::SoftBreak => self.write_raw(" ", out)?,
                Event::HardBreak => self.newline(out)?,
                Event::Rule => self.write_rule(out)?,
                Event::TaskListMarker(checked) => {
                    self.write_raw(if checked { "[x] " } else { "[ ] " }, out)?;
                }
                Event::FootnoteReference(reference) => {
                    self.write_text(&format!("[{reference}]"), out)?
                }
                Event::InlineMath(text) | Event::DisplayMath(text) => {
                    self.write_text(&text, out)?
                }
            }
        }
        self.flush_text(out)?;
        Ok(())
    }

    fn start<W: Write>(&mut self, tag: Tag<'_>, out: &mut W) -> io::Result<()> {
        match tag {
            Tag::Paragraph => self.paragraph_open = true,
            Tag::Heading { .. } => {
                self.blank_line(out)?;
                self.in_heading = true;
                self.style(BOLD, out)?;
            }
            Tag::BlockQuote(_) => self.quote_depth += 1,
            Tag::CodeBlock(kind) => {
                self.blank_line(out)?;
                self.in_code = true;
                self.code.clear();
                self.code_lang = match kind {
                    CodeBlockKind::Fenced(lang) if !lang.is_empty() => Some(lang.to_string()),
                    _ => None,
                };
            }
            Tag::List(start) => {
                if !self.list_stack.is_empty() && !self.text.ends_with('\n') {
                    self.newline(out)?;
                }
                self.list_stack.push(start);
            }
            Tag::Item => self.start_list_item(out)?,
            Tag::Emphasis => self.style(ITALIC, out)?,
            Tag::Strong => self.style(BOLD, out)?,
            Tag::Strikethrough => self.style(DIM, out)?,
            Tag::Link { dest_url, .. } => {
                self.in_link = true;
                self.link_dest = dest_url.to_string();
                self.style(UNDERLINE, out)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn end<W: Write>(&mut self, tag: TagEnd, out: &mut W) -> io::Result<()> {
        match tag {
            TagEnd::Paragraph => {
                self.paragraph_open = false;
                self.newline(out)?;
                self.newline(out)?;
            }
            TagEnd::Heading(_) => {
                self.in_heading = false;
                self.style(RESET, out)?;
                self.newline(out)?;
                self.newline(out)?;
            }
            TagEnd::BlockQuote(_) => self.quote_depth = self.quote_depth.saturating_sub(1),
            TagEnd::CodeBlock => self.finish_code_block(out)?,
            TagEnd::List(_) => {
                self.list_stack.pop();
                self.newline(out)?;
            }
            TagEnd::Item => self.newline(out)?,
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => self.style(RESET, out)?,
            TagEnd::Link => {
                self.style(RESET, out)?;
                if !self.link_dest.is_empty() {
                    self.write_raw(&format!(" ({})", self.link_dest), out)?;
                }
                self.in_link = false;
                self.link_dest.clear();
            }
            _ => {}
        }
        Ok(())
    }

    fn write_text<W: Write>(&mut self, text: &str, out: &mut W) -> io::Result<()> {
        if self.in_code {
            self.code.push_str(text);
            return Ok(());
        }
        if self.in_heading && self.colors {
            self.write_raw(GREEN, out)?;
        }
        self.write_raw(text, out)
    }

    fn write_inline_code<W: Write>(&mut self, text: &str, out: &mut W) -> io::Result<()> {
        self.style(CYAN, out)?;
        self.write_raw(text, out)?;
        self.style(RESET, out)
    }

    fn finish_code_block<W: Write>(&mut self, out: &mut W) -> io::Result<()> {
        self.in_code = false;
        if self.colors {
            let syntax = self
                .code_lang
                .as_deref()
                .and_then(|lang| self.syntax_set.find_syntax_by_token(lang))
                .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
            let mut highlighter = HighlightLines::new(syntax, self.theme);
            for line in LinesWithEndings::from(&self.code) {
                let ranges = highlighter
                    .highlight_line(line, self.syntax_set)
                    .unwrap_or_default();
                write!(
                    out,
                    "    {}",
                    as_24_bit_terminal_escaped(&ranges[..], false)
                )?;
            }
        } else {
            for line in LinesWithEndings::from(&self.code) {
                write!(out, "    {line}")?;
            }
        }
        self.newline(out)?;
        self.newline(out)?;
        Ok(())
    }

    fn start_list_item<W: Write>(&mut self, out: &mut W) -> io::Result<()> {
        let indent = "  ".repeat(self.list_stack.len().saturating_sub(1));
        let marker = match self.list_stack.last_mut() {
            Some(Some(n)) => {
                let marker = format!("{n}. ");
                *n += 1;
                marker
            }
            _ => "• ".to_string(),
        };
        self.write_raw(&indent, out)?;
        self.style(YELLOW, out)?;
        self.write_raw(&marker, out)?;
        self.style(RESET, out)
    }

    fn write_rule<W: Write>(&mut self, out: &mut W) -> io::Result<()> {
        self.blank_line(out)?;
        self.write_raw(&"─".repeat(terminal_width().min(80)), out)?;
        self.newline(out)?;
        self.newline(out)
    }

    fn blank_line<W: Write>(&mut self, out: &mut W) -> io::Result<()> {
        if !self.text.ends_with("\n\n") && !self.text.is_empty() {
            self.newline(out)?;
        }
        Ok(())
    }

    fn newline<W: Write>(&mut self, out: &mut W) -> io::Result<()> {
        self.write_raw("\n", out)
    }

    fn style<W: Write>(&self, style: &str, out: &mut W) -> io::Result<()> {
        if self.colors {
            write!(out, "{style}")?;
        }
        Ok(())
    }

    fn write_raw<W: Write>(&mut self, text: &str, out: &mut W) -> io::Result<()> {
        if self.quote_depth > 0
            && (self.text.is_empty() || self.text.ends_with('\n'))
            && !text.is_empty()
        {
            if self.colors {
                write!(out, "{DIM}{}{RESET} ", "│".repeat(self.quote_depth))?;
            } else {
                write!(out, "{} ", ">".repeat(self.quote_depth))?;
            }
        }
        self.text.push_str(text);
        write!(out, "{text}")
    }

    fn flush_text<W: Write>(&mut self, out: &mut W) -> io::Result<()> {
        out.flush()
    }
}

fn terminal_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|width| *width > 0)
        .unwrap_or(80)
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
    assert!(!p.colors);
}

#[test]
fn nested_list_starts_on_new_line() {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme = Theme::default();
    let mut renderer = TerminalMarkdownRenderer::new(false, &syntax_set, &theme);
    let parser = create_markdown_parser(
        r#"- file
  - flag: -f --file
  - type: string
  - desc: Only run tests from a specific filename
"#,
    );
    let mut buffer = Vec::new();

    renderer.render(parser, &mut buffer).unwrap();

    let rendered = String::from_utf8(buffer).unwrap();
    assert!(rendered.contains("• file\n  • flag: -f --file"));
    assert!(!rendered.contains("• file  • flag: -f --file"));
}
