// Copyright 2020 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

use dialoguer::theme::ColoredTheme;
use dialoguer::{Confirmation, Input, KeyPrompt};
use std::collections::HashSet;
use std::env;
use std::path::Path;

use clap::{crate_name, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand};
use colored::*;

use crate::command::CommandBlock;
use crate::executor::{execute_command, execute_merge_command};
use crate::view;

/// Parse and execute the chosen command.
/// run attempts to ensure that the process does not exit unless there is a panic or clap --help or --version is matched.
/// This enables improved integration testing.
/// Returns exit code, an error string if it should be printed, and if the error should be prefixed with `ERROR`.
/// Inkjet parser created by Brandon Kalinowski See: https://github.com/brandonkal/inkjet
#[inline(never)]
pub fn run(args: Vec<String>, color: bool) -> (i32, String, bool) {
    let (opts, args) = pre_parse(args);
    let color_setting = if color {
        AppSettings::ColoredHelp
    } else {
        AppSettings::ColorNever
    };
    let mut cli_app = App::new(crate_name!())
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::AllowNegativeNumbers)
        .setting(AppSettings::SubcommandRequired)
        .global_setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::VersionlessSubcommands)
        .global_setting(color_setting)
        .version(crate_version!())
        .about("Inkjet is a tool to build interactive CLIs with executable markdown documents.\nInkjet parser created by Brandon Kalinowski\nSee: https://github.com/brandonkal/inkjet")
        .after_help("Run 'inkjet --inkjet-print-all' if you wish to view the complete merged inkjet definition.\nRun 'inkjet COMMAND --help' for more information on a command.")
        .arg(custom_inkfile_path_arg())
        .arg_from_usage(
            "-i --interactive 'Execute the command in the document prompting for arguments'",
        )
        .arg_from_usage("-p --preview 'Preview the command source and exit'");
    let (inkfile, inkfile_path) = crate::loader::find_inkfile(&opts.inkfile_opt);
    if inkfile.is_err() {
        if opts.inkfile_opt.is_empty() || opts.inkfile_opt == "./inkjet.md" {
            // Just log a warning and let the process continue
            eprintln!("{} no inkjet.md found", "WARNING (inkjet):".yellow());
            // If the inkfile can't be found, at least parse for --version or --help
            if let Err(err) = cli_app.get_matches_from_safe(args) {
                let rc = if err.kind == clap::ErrorKind::VersionDisplayed
                    || err.kind == clap::ErrorKind::HelpDisplayed
                {
                    0
                } else {
                    1
                };
                return (rc, err.message, false);
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
    if !mdtxt.contains("inkjet_sort: true") {
        cli_app = cli_app.setting(AppSettings::DeriveDisplayOrder);
    }
    let root_command = match crate::parser::build_command_structure(&mdtxt) {
        Ok(cmd) => cmd,
        Err(err) => {
            return (10, err, true);
        }
    };
    let about_txt = format!(
        "Generated from {}\n\nInkjet parser created by Brandon Kalinowski\nInkjet is a tool to build interactive CLIs with executable markdown documents.\nSee: https://github.com/brandonkal/inkjet\n\n{}",
        inkfile_path, root_command.desc
    );
    cli_app = cli_app.about(about_txt.trim());
    let matches = match build_subcommands(cli_app, &opts, &root_command.subcommands)
        .get_matches_from_safe(args)
    {
        Ok(m) => m,
        Err(err) => {
            let rc = if err.kind == clap::ErrorKind::VersionDisplayed
                || err.kind == clap::ErrorKind::HelpDisplayed
            {
                0
            } else {
                1
            };
            return (rc, err.message, false);
        }
    };

    let mut chosen_cmd = find_command(&matches, &root_command.subcommands)
        .expect("Inkjet: SubcommandRequired failed to work");
    if !chosen_cmd.validation_error_msg.is_empty() {
        return (1, chosen_cmd.validation_error_msg, true);
    }
    let fixed_pwd = !mdtxt.contains("inkjet_fixed_dir: false");

    if opts.interactive {
        let p = view::Printer::new(color, false, &inkfile_path);

        let portion = &mdtxt
            .get(chosen_cmd.start..chosen_cmd.end)
            .expect("Inkjet: portion out of bounds");
        let print_err = p.print_markdown(portion);
        if let Err(err) = print_err {
            return (10, format!("printing markdown: {}", err), true); // cov:include (unusual error)
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
    // We sort here as sorted args are only required for interactive prompts
    chosen_cmd.args.sort_by(|a, b| a.name.cmp(&b.name));
    chosen_cmd.option_flags.sort_by(|a, b| a.name.cmp(&b.name));
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
    for flag in &mut chosen_cmd.option_flags {
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
                    .with_prompt(&format!("{}: Enter option for {}", chosen_cmd.name, name,))
                    .allow_empty(true)
                    .interact()
                    .expect("Inkjet: unable to read option");
                if is_invalid_number(flag.validate_as_number, &rv) {
                    eprintln!("{}: {}", "INVALID".red(), not_number_err_msg(&name));
                    continue;
                } else {
                    break;
                };
            }
            flag.val = rv
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
    let early_exit_modifiers = sset!["-h", "--help", "-V", "--version", "--inkjet-print-all"];
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

fn custom_inkfile_path_arg<'a, 'b>() -> Arg<'a, 'b> {
    // This is needed to prevent clap from complaining about the custom flag check
    // within find_inkfile(). It should be removed once clap 3.x is released.
    // See https://github.com/clap-rs/clap/issues/748
    Arg::with_name("inkfile")
        .help("Path to a different inkfile you want to use")
        .long("inkfile")
        .short("c")
        .takes_value(true)
        .multiple(false)
}
/// Takes a `clap_app` and a parsed root command and recursively builds the CLI application
fn build_subcommands<'a, 'b>(
    mut cli_app: App<'a, 'b>,
    opts: &CustomOpts,
    subcommands: &'a [CommandBlock],
) -> App<'a, 'b> {
    for c in subcommands {
        let mut subcmd = SubCommand::with_name(&c.name)
            .about(c.desc.as_ref())
            .setting(AppSettings::AllowNegativeNumbers);
        if !c.subcommands.is_empty() {
            subcmd = build_subcommands(subcmd, opts, &c.subcommands);
            // If this parent command has no script source, require a subcommand.
            if c.script.source.is_empty() {
                subcmd = subcmd.setting(AppSettings::SubcommandRequired);
            }
        }

        // Add all required and optional arguments
        for a in &c.args {
            let mut arg = Arg::with_name(&a.name).multiple(a.multiple);
            if !opts.preview && !opts.interactive {
                if let Some(def) = &a.default {
                    arg = arg.default_value(def);
                }
            }
            // If we are printing, we can't have required args
            subcmd = subcmd.arg(arg.required(if opts.preview || opts.interactive {
                false
            } else {
                a.required
            }));
        }

        // Add all optional flags
        for f in &c.option_flags {
            let arg = Arg::with_name(&f.name)
                .help(&f.desc)
                .short(&f.short)
                .long(&f.long)
                .takes_value(f.takes_value)
                .multiple(f.multiple);
            subcmd = subcmd.arg(arg);
        }
        if c.name.starts_with('_') {
            subcmd = subcmd.setting(AppSettings::Hidden);
        }
        if !c.aliases.is_empty() {
            let parts = c.aliases.split("//");
            for s in parts {
                subcmd = subcmd.visible_alias(s);
            }
        }
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
                    command = find_command(matches, &c.subcommands)
                        .or_else(|| Some(c.clone()).map(|c| get_command_options(c, matches)));
                    // early exit on validation error (e.g. number required and not supplied)
                    if command
                        .as_ref()
                        .map_or(false, |command| !command.validation_error_msg.is_empty())
                    {
                        return command;
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
fn get_command_options(mut cmd: CommandBlock, matches: &ArgMatches) -> CommandBlock {
    // Check all required args
    for arg in &mut cmd.args {
        arg.val = match matches.values_of(arg.name.clone()) {
            Some(v) => v.collect::<Vec<_>>().join(" "),
            _ => "".to_string(),
        };
    }

    // Check all optional flags
    for flag in &mut cmd.option_flags {
        flag.val = if flag.takes_value {
            // Extract the value
            let raw_value = matches
                .values_of(flag.name.clone())
                .unwrap_or_default()
                .collect::<Vec<_>>()
                .join(" ");

            if is_invalid_number(flag.validate_as_number, &raw_value) {
                cmd.validation_error_msg = not_number_err_msg(&flag.name);
                break;
            }

            raw_value
        } else {
            // Check if the boolean flag is present and set to "true".
            // It's a string since it's set as an environment variable.
            if matches.is_present(flag.name.clone()) {
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
    format!("flag `{}` expects a numerical value", name)
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
        assert_eq!(err_str, "No such file or directory (os error 2)");
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
"#;
        let args = svec!("inkjet", "--inkfile", contents);
        let (rc, _, _) = run(args, false);
        assert_eq!(0, rc);
    }
}
