// Copyright 2020 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

use dialoguer::theme::ColoredTheme;
use dialoguer::{Confirmation, Input, KeyPrompt};
use std::collections::HashSet;
use std::env;
use std::path::Path;

use clap::{Arg, ArgMatches, ColorChoice, Command, builder::styling};
use clap_complete::{Shell, generate};

use crate::command::CommandBlock;
use crate::executor::{execute_command, execute_merge_command};
use crate::{utils, view};

/// Parse and execute the chosen command.
/// run attempts to ensure that the process does not exit unless there is a panic or clap --help or --version is matched.
/// This enables improved integration testing.
/// Returns exit code, an error string if it should be printed, and if the error should be prefixed with `ERROR`.
/// Inkjet parser created by Brandon Kalinowski See: https://github.com/brandonkal/inkjet
pub fn run(args: Vec<String>, color: bool) -> (i32, String, bool) {
    let early_version_detected = match args.get(1) {
        Some(first_arg) => first_arg == "-V" || first_arg == "--version",
        _ => false,
    };
    if early_version_detected {
        return (0, format!("inkjet {}", env!("CARGO_PKG_VERSION")), false);
    }
    let (opts, args) = pre_parse(args);
    let color_setting = if color {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };

    const STYLES: styling::Styles = styling::Styles::styled()
        .header(styling::AnsiColor::Green.on_default().bold())
        .usage(styling::AnsiColor::Green.on_default().bold())
        .literal(styling::AnsiColor::Blue.on_default().bold())
        .placeholder(styling::AnsiColor::Cyan.on_default());

    let mut cli_app = Command::new(env!("CARGO_PKG_NAME"))
        .allow_negative_numbers(true)
        .subcommand_required(true)
        .disable_help_subcommand(true)
        .color(color_setting)
        .styles(STYLES)
        .trailing_var_arg(true)
        .version(env!("CARGO_PKG_VERSION"))
        .about("Inkjet parser created by Brandon Kalinowski\nInkjet is a tool to build interactive CLIs with executable markdown documents.\nSee: https://github.com/brandonkal/inkjet")
        .after_help("Run 'inkjet --inkjet-print-all' if you wish to view the complete merged inkjet definition.\nRun 'inkjet --inkjet-dynamic-completions fish/bash/zsh/powershell' to generate shell completions.\nThis is called dynamically by the global shell completion scripts.\nRun 'inkjet COMMAND --help' for more information on a command.")
        .arg(custom_inkfile_path_arg())
        .arg(
            Arg::new("interactive")
                .short('i')
                .long("interactive")
                .help("Execute the command in the document prompting for arguments")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("preview")
                .short('p')
                .long("preview")
                .help("Preview the command source and exit")
                .action(clap::ArgAction::SetTrue),
        );
    let (inkfile, inkfile_path) = crate::loader::find_inkfile(&opts.inkfile_opt);
    if inkfile.is_err() {
        if opts.inkfile_opt.is_empty() || opts.inkfile_opt == "./inkjet.md" {
            // Just log a warning and let the process continue
            eprintln!("{} no inkjet.md found", utils::warn_msg());

            // If the inkfile can't be found, at least parse for --version or --help
            if let Err(err) = cli_app.try_get_matches_from(args) {
                let rc = if err.kind() == clap::error::ErrorKind::DisplayVersion
                    || err.kind() == clap::error::ErrorKind::DisplayHelp
                {
                    0
                } else {
                    1
                };
                return (rc, err.to_string(), false);
            };
            return (10, "No argument match found".to_string(), true); // cov:ignore (won't be called if help is parsed)
        } else {
            return (
                10,
                format!("specified inkfile \"{}\" not found", opts.inkfile_opt),
                true,
            ); // won't be called if help is parsed
        }
    }
    let mut mdtxt = inkfile.unwrap();

    // If import directive is included,
    // merge all files first and then parse resulting text output
    if mdtxt.contains("inkjet_import: all") {
        match execute_merge_command(&inkfile_path) {
            Ok(txt) => {
                mdtxt = txt;
            }

            Err(err) /* cov:include */ => {
                return (10, err, true);
            }
        };
    }
    if opts.print_all {
        return (0, mdtxt, false);
    }
    // By default subcommands in the help output are listed in the same order
    // they are defined in the markdown file. Users can define this directive
    // for alphabetical sort.
    let alphabetical_sort = mdtxt.contains("inkjet_sort: true");

    let in_completions_mode =
        args.len() > 2 && args.get(1).unwrap_or(&String::from("")) == "inkjet-dynamic-completions";
    let root_command = match crate::parser::build_command_structure(&mdtxt, !in_completions_mode) {
        Ok(cmd) => cmd,
        Err(err) => {
            return (10, err, true);
        }
    };
    let about_txt = format!(
        "Generated from {}\n\nInkjet parser created by Brandon Kalinowski\nInkjet is a tool to build interactive CLIs with executable markdown documents.\nSee: https://github.com/brandonkal/inkjet\n\n{}",
        inkfile_path, root_command.desc
    );
    cli_app = cli_app.about(about_txt.trim().to_string());
    cli_app = build_subcommands(
        cli_app,
        &opts,
        root_command.subcommands.clone(),
        alphabetical_sort,
    );

    // Manual arg parsing for inkjet-dynamic-completions because it should not be required
    #[allow(clippy::indexing_slicing)]
    if in_completions_mode {
        let shell = match args[2].as_str() {
            "bash" => Shell::Bash,
            "fish" => Shell::Fish,
            "zsh" => Shell::Zsh,
            "powershell" => Shell::PowerShell,
            _ => {
                return (1, format!("Unsupported shell: {}", args[2]), false);
            }
        };
        let mut buffer: Vec<u8> = Vec::new();
        generate(shell, &mut cli_app, "inkjet", &mut buffer);
        let mut output = String::from_utf8_lossy(&buffer).into_owned();
        if args[2].as_str() == "bash" {
            output = output
                .lines()
                .filter(|line| !line.contains("complete"))
                .collect::<Vec<&str>>()
                .join("\n")
        } else if args[2].as_str() == "fish" {
            // There is a bug in clap where it adds help commands to completions.
            // So we filter it out here.
            output = output
                .lines()
                .filter(|line| !line.contains("-a \"help\""))
                .collect::<Vec<&str>>()
                .join("\n")
        }
        return (0, output, false); // Exit after generating completion
    }

    let matches = match cli_app.clone().try_get_matches_from(args) {
        Ok(m) => m,
        Err(err) => {
            let rc = if err.kind() == clap::error::ErrorKind::DisplayVersion
                || err.kind() == clap::error::ErrorKind::DisplayHelp
            {
                0
            } else {
                1
            };
            return (rc, err.to_string(), false);
        }
    };

    let mut chosen_cmd = find_command(&matches, &root_command.subcommands)
        .expect("Inkjet: SubcommandRequired failed to work");
    if !chosen_cmd.validation_error_msg.is_empty() {
        return (1, chosen_cmd.validation_error_msg, true);
    }
    let fixed_pwd = !mdtxt.contains("inkjet_fixed_dir: false");

    if opts.interactive {
        let p = view::Printer::new(color, &inkfile_path);

        let portion = &mdtxt
            .get(chosen_cmd.start..chosen_cmd.end)
            .expect("Inkjet: portion out of bounds");
        let print_err = p.print_markdown(portion);
        if let Err(err) = print_err {
            return (10, format!("printing markdown: {err}"), true); // cov:include (unusual error)
        }
        eprintln!();
        let (picked_cmd, exit_code, err_str) =
            interactive_params(chosen_cmd, &inkfile_path, color, fixed_pwd);
        if picked_cmd.is_none() {
            return (exit_code, err_str, true); // cov:include (skipped command)
        }
        chosen_cmd = picked_cmd.unwrap();
    }
    match execute_command(chosen_cmd, &inkfile_path, opts.preview, color, fixed_pwd) {
        Some(result) => match result {
            Ok(status) => {
                if let Some(code) = status.code() {
                    (code, "".to_string(), false)
                } else {
                    (0, "".to_string(), false) // cov:ignore (unusual)
                }
            }
            Err(err) => (10, err.to_string(), false),
        },
        _ => (0, "".to_string(), false),
    }
}

/// Prompt for missing parameters interactively.
#[inline(never)]
fn interactive_params(
    mut chosen_cmd: CommandBlock,
    inkfile_path: &str,
    color: bool,
    fixed_dir: bool,
) -> (Option<CommandBlock>, i32, String) {
    // cov:begin-include
    loop {
        let rv = KeyPrompt::with_theme(&ColoredTheme::default())
            .with_text(&format!("Execute step {}?", chosen_cmd.name))
            .items(&['y', 'n', 'p'])
            .default(0)
            .interact()
            .expect("Inkjet: unable to read response");
        if rv == 'y' {
            break;
        } else if rv == 'p' {
            match execute_command(chosen_cmd.clone(), inkfile_path, true, color, fixed_dir) {
                Some(result) => {
                    match result {
                        Ok(exit_status) => {
                            if exit_status.success() {
                                eprintln!(); // empty space
                                continue;
                            } else {
                                return (None, exit_status.code().unwrap_or(10), "unable to preview command (perhaps bat returned non-zero status)".to_string());
                            }
                        }
                        Err(err) => {
                            return (None, 10, err.to_string());
                        }
                    }
                }
                _ => {
                    return (None, 0, "".to_string());
                }
            }
        } else {
            eprintln!("Skipping command {}", chosen_cmd.name);
            return (None, 0, "".to_string());
        }
    }
    for flag in &mut chosen_cmd.named_flags {
        if !flag.takes_value {
            if flag.name == "verbose" {
                continue;
            }
            if flag.val != "true" {
                let rv: bool = Confirmation::with_theme(&ColoredTheme::default())
                    .with_text(&format!("{}: Set {} option?", chosen_cmd.name, flag.name))
                    .default(false)
                    .interact()
                    .expect("Inkjet: unable to confirm option");
                if rv {
                    flag.val = "true".to_string();
                }
            }
        } else if flag.val.is_empty() {
            let mut rv: String;
            loop {
                let name = flag.name.clone();
                rv = Input::with_theme(&ColoredTheme::default())
                    .with_prompt(&format!(
                        "{}: Enter option for {}{}",
                        chosen_cmd.name,
                        name,
                        if flag.required { " *" } else { "" }
                    ))
                    .allow_empty(!flag.required)
                    .interact()
                    .expect("Inkjet: unable to read option");
                if !flag.choices.is_empty() && !flag.choices.contains(&rv) {
                    eprintln!(
                        "{}: {} flag expects one of {:?}",
                        utils::invalid_msg(),
                        flag.name,
                        flag.choices
                    );
                    continue;
                }
                if is_invalid_number(flag.validate_as_number, &rv) {
                    eprintln!("{}: {}", utils::invalid_msg(), not_number_err_msg(&name));
                    continue;
                } else {
                    break;
                };
            }
            flag.val = rv
        }
    }
    for arg in chosen_cmd.args.iter_mut() {
        if arg.val.is_empty() {
            let rv: String = Input::with_theme(&ColoredTheme::default())
                .with_prompt(&format!(
                    "{}: Enter value for {}{}",
                    chosen_cmd.name,
                    arg.name,
                    if arg.required { " *" } else { "" },
                ))
                .allow_empty(!arg.required)
                .default(arg.default.clone())
                .interact()
                .expect("Inkjet: unable to read input");
            arg.val = rv
        }
    }
    (Some(chosen_cmd), 0, "".to_string())
    // cov:end-include
}

/// Creates vector of strings, Vec<String>
macro_rules! svec {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}
/// Creates HashSet<String> from string literals
macro_rules! sset {
  ($($x:expr),*) => {{
    let _v = svec![$($x.to_string()),*];
    let hash_set: HashSet<String> = _v.iter().cloned().collect();
    hash_set
  }}
}

#[derive(Default, Debug)]
struct CustomOpts {
    interactive: bool,
    preview: bool,
    inkfile_opt: String,
    print_all: bool,
}

/// We must parse flags first to handle global flags and implicit defaults
fn pre_parse(mut args: Vec<String>) -> (CustomOpts, Vec<String>) {
    let mut opts = CustomOpts::default();
    let early_exit_modifiers = sset![
        "-h",
        "--help",
        "-V",
        "--version",
        "--inkjet-print-all",
        "--inkjet-dynamic-completions"
    ];
    // Loop through all args and parse
    let mut inkfile_index = 1000;
    // If the first argument is a markdown file or '-' assume it is a inkfile arg
    // This allows us to use it as an interpreter without specifying '--inkfile'
    #[allow(clippy::indexing_slicing)]
    {
        if args.len() > 1
            && (args[1] == "-" || (args[1].ends_with(".md") && !args[1].starts_with('-')))
        {
            args.insert(1, "--inkfile".to_string());
        }
    }
    let mut default_index = 0;

    for i in 1..args.len() {
        #[allow(clippy::indexing_slicing)]
        let arg = &args[i];
        // we keep track of if default should be inserted. iff not 0, we insert it.
        // if -1 we insert it at the end.
        if i == inkfile_index {
            opts.inkfile_opt = canonical_path(arg);
            if i == args.len() - 1 {
                default_index = 1000; // prevent duplicate default insertions
                args.insert(i + 1, "default".to_string());
                break;
            }
        } else if arg == "-i" || arg == "--interactive" {
            opts.interactive = true;
        } else if arg.starts_with("--inkfile") || arg.starts_with("-c") {
            if let Some(eq_idx) = arg.find('=') {
                #[allow(clippy::indexing_slicing)]
                let part2 = &arg[(eq_idx + 1)..];
                opts.inkfile_opt = canonical_path(part2);
                inkfile_index = 999; // we've found it. No need to insert
            } else {
                inkfile_index = i + 1
            }
        } else if arg == "--preview" || arg == "-p" {
            if !opts.preview {
                opts.preview = true;
            }
        } else if arg == "--inkjet-print-all" {
            opts.print_all = true;
            default_index = 1000;
            break;
        } else if arg.ends_with(".md") && inkfile_index == 1000 {
            // we found a markdown filename without it being proceeded by `--inkfile`
            // we will insert that after the loop if required.
            opts.inkfile_opt = canonical_path(arg);
            inkfile_index = i
        // if it is not a flag or early exit:
        } else if !arg.starts_with('-') || early_exit_modifiers.contains(arg) {
            default_index = 1000;
            break; // no more parsing to do as a subcommand has been called
        } else if arg == "-" {
            continue; // stdin file input
        } else {
            // This may be a flag for the default command.
            default_index = i;
            // insert modifies the length, but this is ok because we break.
            break;
        }
    }
    if default_index <= args.len() {
        if default_index == 0 {
            args.push("default".to_string());
        } else {
            args.insert(default_index, "default".to_string());
        }
    }
    // if it is within the range, check to see if we need to insert the flag.
    if inkfile_index <= args.len() {
        let flag = args.get(inkfile_index - 1).unwrap();
        if !(flag == "-c" || flag == "--inkfile") {
            args.insert(inkfile_index, "--inkfile".to_string());
        }
    }
    (opts, args)
}

// converts a given path str to a canonical path String
fn canonical_path(p: &str) -> String {
    Path::new(p)
        .to_str()
        .expect("Inkjet: could not canonicalize path")
        .to_string()
}

fn custom_inkfile_path_arg() -> Arg {
    // This is needed to prevent clap from complaining about the custom flag check
    // within find_inkfile(). It should be removed once clap 3.x is released.
    // See https://github.com/clap-rs/clap/issues/748
    Arg::new("inkfile")
        .help("Path to a different inkfile you want to use")
        .long("inkfile")
        .short('c')
        .value_name("FILE")
        .action(clap::ArgAction::Set)
}
/// Helper function to build a Command from a CommandBlock
fn build_command_from_block(cmd_block: CommandBlock, opts: &CustomOpts, sort: bool) -> Command {
    let name = cmd_block.name;
    let desc = cmd_block.desc;
    let args = cmd_block.args;
    let named_flags = cmd_block.named_flags;
    let aliases = cmd_block.aliases;
    let script_source = cmd_block.script.source;
    let starts_with_underscore = name.starts_with('_');
    let subcommands = cmd_block.subcommands;

    // Create a new owned Command
    let mut cmd = Command::new(name).about(desc).allow_negative_numbers(true);

    // Process subcommands recursively
    if !subcommands.is_empty() {
        // Pass ownership of the subcommands
        cmd = build_subcommands(cmd, opts, subcommands, sort);
        // If this parent command has no script source, require a subcommand.
        if script_source.is_empty() {
            cmd = cmd.subcommand_required(true);
        }
    }

    // Add all positional arguments
    for a in args {
        // Convert to owned strings to satisfy 'static lifetime requirement
        let arg_name = a.name.clone();
        let mut arg = Arg::new(arg_name);
        if a.multiple {
            arg = arg.action(clap::ArgAction::Append);
        } else {
            arg = arg.action(clap::ArgAction::Set);
        }
        if !opts.preview && !opts.interactive {
            if let Some(def) = &a.default {
                // Convert to owned string
                let default_value = def.clone();
                arg = arg.default_value(default_value);
            }
            // Handle "extras" arg to collect everything after the --
            if a.last {
                arg = arg.last(true);
            }
        }
        // If we are printing, we can't have required args
        arg = arg.required(if opts.preview || opts.interactive {
            false
        } else {
            a.required
        });
        cmd = cmd.arg(arg);
    }

    // Add all named flags
    for f in named_flags {
        // Convert to owned strings to satisfy 'static lifetime requirement
        let flag_name = f.name.clone();
        let flag_desc = f.desc.clone();
        let flag_long = f.long.clone();

        let mut arg = Arg::new(flag_name)
            .help(flag_desc)
            .long(flag_long)
            .required(if opts.preview || opts.interactive {
                false
            } else {
                f.required
            });

        if !f.short.is_empty() {
            arg = arg.short(f.short.chars().next().unwrap_or('?'));
        }

        if f.takes_value {
            if f.multiple {
                arg = arg.action(clap::ArgAction::Append);
            } else {
                arg = arg.action(clap::ArgAction::Set);
            }
        } else {
            arg = arg.action(clap::ArgAction::SetTrue);
        }

        cmd = cmd.arg(arg);
    }

    if starts_with_underscore {
        cmd = cmd.hide(true);
    }

    if !aliases.is_empty() {
        // Split the aliases string and convert each alias to an owned string
        for s in aliases.split("//") {
            let alias = s.to_string();
            cmd = cmd.visible_alias(alias);
        }
    }

    cmd
}

/// Takes a `clap_app` and a parsed root command and recursively builds the CLI application
fn build_subcommands(
    mut cli_app: Command,
    opts: &CustomOpts,
    subcommands: Vec<CommandBlock>,
    sort: bool,
) -> Command {
    for c in subcommands {
        // Build a new Command from the CommandBlock
        let mut subcmd = build_command_from_block(c, opts, sort);
        if sort {
            subcmd = subcmd.display_order(0);
        }

        // Add the subcommand to the parent command
        // In clap v4, subcommand takes ownership of the Command
        cli_app = cli_app.subcommand(subcmd);
    }

    cli_app
}
/// finds the CommandBlock to execute based on supplied args. If the user input fails validation then
/// the validation_error_msg property will be non-empty
fn find_command(matches: &ArgMatches, subcommands: &[CommandBlock]) -> Option<CommandBlock> {
    let mut command = None;
    // The child subcommand that was used
    if let Some(subcommand_name) = matches.subcommand_name() {
        if let Some(matches) = matches.subcommand_matches(subcommand_name) {
            for c in subcommands {
                if c.name == subcommand_name {
                    // Check if a subcommand was called, otherwise return this command
                    let c_clone = c.clone();
                    command = find_command(matches, &c_clone.subcommands)
                        .or_else(|| Some(c_clone.clone()).map(|c| embed_arg_values(c, matches)));
                    // early exit on validation error (e.g. number required and not supplied)
                    if let Some(ref cmd) = command {
                        if !cmd.validation_error_msg.is_empty() {
                            return command;
                        }
                    }
                }
            }
        }
    }
    command
}

/// For a given set of matches, apply those arg values to the chosen CommandBlock
/// returns the CommandBlock or an error string on Error (number invalid)
/// If a flag validation error occurs, the validation_error_msg key will be mutated and parsing will stop.
fn embed_arg_values(mut cmd: CommandBlock, matches: &ArgMatches) -> CommandBlock {
    // Check all required args
    for arg in &mut cmd.args {
        arg.val = match matches.get_many::<String>(&arg.name) {
            Some(values) => values.map(|s| s.as_str()).collect::<Vec<_>>().join(" "),
            _ => "".to_string(),
        };
    }

    // Check all named flags
    for flag in &mut cmd.named_flags {
        flag.val = if flag.takes_value {
            // Extract the value
            let raw_value = match matches.get_many::<String>(&flag.name) {
                Some(values) => values.map(|s| s.as_str()).collect::<Vec<_>>().join(" "),
                _ => "".to_string(),
            };
            if !flag.choices.is_empty()
                && !raw_value.is_empty()
                && !flag.choices.contains(&raw_value)
            {
                cmd.validation_error_msg = format!(
                    "{}: {} flag expects one of {:?}",
                    utils::invalid_msg(),
                    flag.name,
                    flag.choices
                );
                break;
            }

            if is_invalid_number(flag.validate_as_number, &raw_value) {
                cmd.validation_error_msg = not_number_err_msg(&flag.name);
                break;
            }

            raw_value
        } else {
            // Check if the boolean flag is present and set to "true".
            // It's a string since it's set as an environment variable.
            if *matches.get_one::<bool>(&flag.name).unwrap_or(&false) {
                "true".to_string()
            } else {
                "".to_string()
            }
        };
    }
    cmd
}
/// returns true if flag is set and the string should parse as number and does not
fn is_invalid_number(is_num: bool, raw_value: &str) -> bool {
    if !is_num || raw_value.is_empty() {
        return false;
    }
    // Try converting to an integer or float to validate it
    raw_value.parse::<isize>().is_err() && raw_value.parse::<f32>().is_err()
}

fn not_number_err_msg(name: &str) -> String {
    format!("flag `{name}` expects a numerical value")
}

#[cfg(test)]
mod runner_tests {
    use super::*;

    #[test]
    fn fake_language() {
        let contents = r#"
## default
```fake
echo "This should not run"
```
        "#;
        let args = svec!("inkjet", "--inkfile", contents);
        let (rc, err_str, _) = run(args, false);
        assert_eq!(rc, 10);

        #[cfg(windows)]
        let no_file_error = "program not found";

        #[cfg(not(windows))]
        let no_file_error = "No such file or directory (os error 2)";

        assert_eq!(err_str, no_file_error);
    }

    #[test]
    fn handles_positional_args() {
        let contents = r#"
## build//default

> A test to check implicit execution of default when calling inkjet without arguments.

```
echo "expected output"
```

## echo (name) (optional=default) (not_required?) -- (extras...?)

> Echo something interactively

**OPTIONS**

- flag: --num |number| A number
- flag: --required -r |string| required This must be specified
- flag: --any |string| Anything you want

```bash
echo "Hello $name! Optional arg is \"$optional\". Number is \"$num\". Required is \"$required\". Any is \"$any\". extras is \"$extras\""
```

## extras (extra...?)

> Test multiple optional values for extra

```
echo "Hello $extra"
```
        "#;
        let args = svec!(
            "inkjet",
            "--inkfile",
            contents,
            "echo",
            "test_runner",
            "--required",
            "req",
            "--",
            "last_arg"
        );
        let (rc, err_str, _) = run(args, false);
        assert_eq!(rc, 0);
        assert_eq!(err_str, "");
    }

    #[test]
    fn numbers() {
        let is_f = is_invalid_number(false, "string");
        assert!(!is_f);
        let is_f = is_invalid_number(true, "42");
        assert!(!is_f);
        let is_t = is_invalid_number(true, "abc");
        assert!(is_t);
        not_number_err_msg("flag");
    }

    #[test]
    fn modify_args() {
        let (_, a) = pre_parse(svec!("inkjet", "tests/simple_case/inkjet.md", "-p"));
        assert_eq!(
            a,
            svec!(
                "inkjet",
                "--inkfile",
                "tests/simple_case/inkjet.md",
                "-p",
                "default"
            )
        );
    }

    #[test]
    fn no_leak_to_positional_args() {
        let (_, a) = pre_parse(svec!("inkjet", "tests/simple_case/inkjet.md"));
        assert_eq!(
            a,
            svec!(
                "inkjet",
                "--inkfile",
                "tests/simple_case/inkjet.md",
                "default"
            )
        );
    }

    #[test]
    fn modify_args2() {
        let x2 = svec!("inkjet", "-p", "--inkfile", "file.txt", "-", "something");
        let (_, o) = pre_parse(x2.clone());
        assert_eq!(o, x2);
    }

    #[test]
    fn preview() {
        let args = svec!["inkjet", "tests/simple_case/inkjet.md", "-p"];
        run(args, false);
    }

    #[test]
    fn inkfile_with_equals() {
        let (o, a) = pre_parse(svec!("inkjet", "--inkfile=tests/simple_case/inkjet.md"));
        assert_eq!(
            a,
            svec!("inkjet", "--inkfile=tests/simple_case/inkjet.md", "default")
        );
        assert!(o.inkfile_opt.contains("simple_case/inkjet.md"));
    }

    #[test]
    fn implicit_inkfile_flag() {
        let (o, a) = pre_parse(svec!("inkjet", "tests/simple_case/inkjet.md", "--flag"));
        assert_eq!(
            a,
            svec!(
                "inkjet",
                "--inkfile",
                "tests/simple_case/inkjet.md",
                "default",
                "--flag"
            )
        );
        assert!(o.inkfile_opt.contains("simple_case/inkjet.md"));
    }

    #[test]
    fn implicit_inkfile_param() {
        let (o, a) = pre_parse(svec!(
            "inkjet",
            "-p",
            "tests/simple_case/inkjet.md",
            "--flag"
        ));
        assert_eq!(
            a,
            svec!(
                "inkjet",
                "-p",
                "--inkfile",
                "tests/simple_case/inkjet.md",
                "default",
                "--flag"
            )
        );
        assert!(o.inkfile_opt.contains("simple_case/inkjet.md"));
    }

    #[test]
    fn inkfile_is_contents() {
        let contents = r#"
## default
```
echo "This is the default"
```
```powershell
param (
    $in = $env:val
)
Write-Output "Value: $in"
```
"#;
        let args = svec!("inkjet", "--inkfile", contents);
        let (rc, _, _) = run(args, false);
        assert_eq!(0, rc);
    }
}
