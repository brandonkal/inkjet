//! Make your markdown executable with inkjet, the interactive CLI task runner
#![warn(clippy::indexing_slicing)]
#![warn(missing_docs)]
/// The `inkjet::command` module holds CommandBlock and its types
pub mod command;
/// The `inkjet::executor` module contains the implementations to prepare and execute a CommandBlock
pub mod executor;
/// The `inkjet::loader` module contains the implementations to read and inkfile from disk or stdin prior to parsing.
pub mod loader;
/// The `inkjet::parser` module is responsible for parsing a markdown string and returning a CommandBlock tree.
pub mod parser;
/// The `inkjet::runner` module contains the main inkjet CLI logic. Call `inkjet::runner::run` with args and color setting.
pub mod runner;
/// The `inkjet::view` module contains the implementation for printing markdown to the terminal. It is used for interactive mode.
pub mod view;
