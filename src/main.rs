//! Make your markdown executable with inkjet, the interactive CLI task runner
#![warn(clippy::indexing_slicing)]
use dialoguer::theme::ColoredTheme;
use dialoguer::{Confirmation, Input, KeyPrompt};
use std::collections::HashSet;
use std::env;
use std::path::Path;

use clap::{crate_name, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand};
use colored::*;

use inkjet::command::Command;
use inkjet::executor::{execute_command, execute_merge_command};
use inkjet::view;

fn main() {
    let color = env::var_os("NO_COLOR").is_none();
    let args = env::args().collect();
    let (rc, err_str, prefix) = run(args, color);
    if !err_str.is_empty() {
        if prefix {
            eprintln!("{}: {}", "ERROR".red(), err_str);
        } else {
            eprintln!("{}", err_str);
        }
    }
    std::process::exit(rc);
}

/// Parse and execute the chosen command.
/// run attempts to ensure that the process does not exit unless there is a panic or clap --help or --version is matched.
/// This enables improved integration testing.
/// Returns exit code, an error string if it should be printed, and if the error should be prefixed with `ERROR`.
fn run(args: Vec<String>, color: bool) -> (i32, String, bool) {
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
        .global_setting(color_setting)
        .version(crate_version!())
        .about("Inkjet is a tool to build interactive CLIs with executable markdown.\nInkjet parser created by Brandon Kalinowski")
        .arg(custom_inkfile_path_arg())
        .arg_from_usage(
            "-i --interactive 'Execute the command in the document prompting for arguments'",
        )
        .arg_from_usage("-p --preview 'Preview the command source and exit'");
    let (inkfile, inkfile_path) = inkjet::loader::find_inkfile(&opts.inkfile_opt);
    if inkfile.is_err() {
        if opts.inkfile_opt == "" || opts.inkfile_opt == "./inkjet.md" {
            // Just log a warning and let the process continue
            eprintln!("{} no inkjet.md found", "WARNING:".yellow());
            // If the inkfile can't be found, at least parse for --version or --help
            if let Err(err) = cli_app.get_matches_from_safe(args) {
                return (1, err.message, false);
            };
            return (10, "No argument match found".to_string(), true); // won't be called if help is parsed
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
    if mdtxt.contains("inkjet_import:") {
        match execute_merge_command(&inkfile_path) {
            Ok(txt) => {
                mdtxt = txt;
            }
            Err(err) => {
                return (10, err, true);
            }
        };
    }
    if opts.print_all {
        print!("{}", mdtxt);
        return (0, "".to_string(), true);
    }
    // By default subcommands in the help output are listed in the same order
    // they are defined in the markdown file. Users can define this directive
    // for alphabetical sort.
    if !mdtxt.contains("inkjet_sort: true") {
        cli_app = cli_app.setting(AppSettings::DeriveDisplayOrder);
    }
    let root_command = match inkjet::parser::build_command_structure(&mdtxt) {
        Ok(cmd) => cmd,
        Err(err) => {
            return (10, err, true);
        }
    };
    let about_txt = format!(
        "Generated from {}\nInkjet parser created by Brandon Kalinowski\n\n{}",
        inkfile_path, root_command.desc
    );
    cli_app = cli_app.about(about_txt.trim());
    let matches = match build_subcommands(cli_app, &opts, &root_command.subcommands)
        .get_matches_from_safe(args)
    {
        Ok(m) => m,
        Err(err) => {
            return (1, err.message, false);
        }
    };

    let mut chosen_cmd = find_command(&matches, &root_command.subcommands)
        .expect("SubcommandRequired failed to work");
    if !chosen_cmd.validation_error_msg.is_empty() {
        return (1, chosen_cmd.validation_error_msg, true);
    }
    let fixed_pwd = !mdtxt.contains("inkjet_fixed_dir: false");

    if opts.interactive {
        let p = view::Printer::new(color, false, &inkfile_path);

        let portion = &mdtxt
            .get(chosen_cmd.start..chosen_cmd.end)
            .expect("portion out of bounds");
        let print_err = p.print_markdown(&portion);
        if let Err(err) = print_err {
            return (10, format!("printing markdown: {}", err), true);
        }
        eprintln!();
        let (picked_cmd, exit_code, err_str) =
            interactive_params(chosen_cmd, &inkfile_path, color, fixed_pwd);
        if picked_cmd.is_none() {
            return (exit_code, err_str, true);
        }
        chosen_cmd = picked_cmd.unwrap();
    }
    match execute_command(chosen_cmd, &inkfile_path, opts.preview, color, fixed_pwd) {
        Some(result) => match result {
            Ok(status) => {
                if let Some(code) = status.code() {
                    (code, "".to_string(), false)
                } else {
                    (0, "".to_string(), false)
                }
            }
            Err(err) => (10, err.to_string(), false),
        },
        _ => (0, "".to_string(), false),
    }
}

/// Prompt for missing parameters interactively.
fn interactive_params(
    mut chosen_cmd: Command,
    inkfile_path: &str,
    color: bool,
    fixed_dir: bool,
) -> (Option<Command>, i32, String) {
    loop {
        let rv = KeyPrompt::with_theme(&ColoredTheme::default())
            .with_text(&format!("Execute step {}?", chosen_cmd.name))
            .items(&['y', 'n', 'p'])
            .default(0)
            .interact()
            .expect("unable to read response");
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
        if arg.val == "" {
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
                .expect("unable to read input");
            arg.val = rv
        }
    }
    for flag in &mut chosen_cmd.option_flags {
        if !flag.takes_value {
            if flag.name == "verbose" {
                break;
            }
            if flag.val != "true" {
                let rv: bool = Confirmation::with_theme(&ColoredTheme::default())
                    .with_text(&format!("{}: Set {} option?", chosen_cmd.name, flag.name))
                    .default(false)
                    .interact()
                    .expect("unable to confirm option");
                if rv {
                    flag.val = "true".to_string();
                }
            }
        } else if flag.val == "" {
            let mut rv: String;
            loop {
                let name = flag.name.clone();
                rv = Input::with_theme(&ColoredTheme::default())
                    .with_prompt(&format!("{}: Enter option for {}", chosen_cmd.name, name,))
                    .allow_empty(true)
                    .interact()
                    .expect("unable to read option");
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
        if args.len() > 1 && (args[1] == "-" || args[1].ends_with(".md")) {
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
                // No default was called.
                args.insert(i + 1, "default".to_string());
                break;
            }
        } else if arg == "-i" || arg == "--interactive" {
            opts.interactive = true;
        } else if arg == "--inkfile" || arg == "-c" {
            if let Some(idx) = arg.find('=') {
                #[allow(clippy::indexing_slicing)]
                let part2 = &arg[(idx + 1)..];
                opts.inkfile_opt = canonical_path(part2);
                inkfile_index = 999; // we've found it
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
        } else if arg.ends_with(".md") && inkfile_index != 1000 {
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
    if inkfile_index < args.len() {
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
        .expect("could not canonicalize path")
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
    subcommands: &'a [Command],
) -> App<'a, 'b> {
    for c in subcommands {
        let mut subcmd = SubCommand::with_name(&c.name)
            .about(c.desc.as_ref())
            .setting(AppSettings::AllowNegativeNumbers);
        if !c.subcommands.is_empty() {
            subcmd = build_subcommands(subcmd, opts, &c.subcommands);
            // If this parent command has no script source, require a subcommand.
            if c.script.source == "" {
                subcmd = subcmd.setting(AppSettings::SubcommandRequired);
            }
        }

        // Add all required and optional arguments
        for a in &c.args {
            let arg = Arg::with_name(&a.name);
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
        if c.aliases != "" {
            let parts = c.aliases.split("//");
            for s in parts {
                subcmd = subcmd.visible_alias(&*s);
            }
        }
        cli_app = cli_app.subcommand(subcmd);
    }

    cli_app
}
/// finds the Command to execute based on supplied args. If the user input fails validation then
/// the validation_error_msg property will be non-empty
fn find_command(matches: &ArgMatches, subcommands: &[Command]) -> Option<Command> {
    let mut command = None;
    // The child subcommand that was used
    if let Some(subcommand_name) = matches.subcommand_name() {
        if let Some(matches) = matches.subcommand_matches(subcommand_name) {
            for c in subcommands {
                if c.name == subcommand_name {
                    // Check if a subcommand was called, otherwise return this command
                    command = find_command(matches, &c.subcommands)
                        .or_else(|| Some(c.clone()).map(|c| get_command_options(c, &matches)));
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
/// returns the Command or an error string on Error (number invalid)
/// If a flag validation error occurs, the validation_error_msg key will be mutated and parsing will stop.
fn get_command_options(mut cmd: Command, matches: &ArgMatches) -> Command {
    // Check all required args
    for arg in &mut cmd.args {
        arg.val = match matches.value_of(arg.name.clone()) {
            Some(v) => v,
            _ => "",
        }
        .to_string();
    }

    // Check all optional flags
    for flag in &mut cmd.option_flags {
        flag.val = if flag.takes_value {
            // Extract the value
            let raw_value = matches
                .value_of(flag.name.clone())
                .unwrap_or("")
                .to_string();

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
    if !is_num || raw_value == "" {
        return false;
    }
    // Try converting to an integer or float to validate it
    raw_value.parse::<isize>().is_err() && raw_value.parse::<f32>().is_err()
}

fn not_number_err_msg(name: &str) -> String {
    format!("flag `{}` expects a numerical value", name)
}

#[cfg(test)]
mod main_tests {
    use super::*;
    #[test]
    fn numbers() {
        let is_f = is_invalid_number(false, "string");
        assert_eq!(is_f, false);
        let is_f = is_invalid_number(true, "42");
        assert_eq!(is_f, false);
        let is_t = is_invalid_number(true, "abc");
        assert_eq!(is_t, true);
        not_number_err_msg("flag");
    }
    #[test]
    fn modify_args() {
        let (_, o) = pre_parse(svec!("inkjet", "tests/simple_case/inkjet.md", "-p"));
        assert_eq!(
            o,
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
    // #[test]
    // fn interactive() {
    //     let args = svec!["inkjet", "tests/simple_case/inkjet.md", "-i"];
    //     run(args, false);
    // }
}
