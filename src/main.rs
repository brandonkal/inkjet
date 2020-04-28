use dialoguer::theme::ColoredTheme;
use dialoguer::{Confirmation, Input, KeyPrompt};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;

use clap::{crate_name, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand};
use colored::*;

use inkjet::command::Command;
use inkjet::executor::execute_command;
use inkjet::view;

fn main() {
    let color = env::var_os("NO_COLOR").is_none();
    let color_setting = if color {
        AppSettings::ColoredHelp
    } else {
        AppSettings::ColorNever
    };
    let mut cli_app = App::new(crate_name!())
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::AllowNegativeNumbers)
        .setting(AppSettings::SubcommandRequired)
        .setting(color_setting)
        .version(crate_version!())
        .about("Inkjet parser created by Brandon Kalinowski")
        .arg(custom_inkfile_path_arg())
        .arg_from_usage(
            "-i --interactive 'Execute the command in the document prompting for arguments'",
        )
        .arg_from_usage("-p --preview 'Preview the command source and exit'");

    let (opts, args) = pre_parse(env::args().collect());

    let (inkfile, inkfile_path) = find_inkfile(&opts.inkfile_path);
    if inkfile.is_err() {
        // If the inkfile can't be found, at least parse for --version or --help
        cli_app.get_matches();
        return;
    }
    let mdtxt = inkfile.unwrap();
    let about_txt = format!(
        "Generated from the {} file.\nInkjet parser created by Brandon Kalinowski",
        inkfile_path
    );

    cli_app = cli_app.about(about_txt.as_str());

    let root_command = inkjet::parser::build_command_structure(mdtxt.clone());
    let matches = build_subcommands(cli_app, color_setting, &opts, &root_command.subcommands)
        .get_matches_from(args);
    let mut chosen_cmd = find_command(&matches, &root_command.subcommands)
        .expect("SubcommandRequired failed to work");

    if opts.interactive {
        let p = view::Printer::new(color, false, &inkfile_path);
        let portion = &mdtxt[chosen_cmd.start..chosen_cmd.end];
        let print_err = p.print_markdown(&portion);
        if let Err(err) = print_err {
            eprintln!("{} printing markdown: {}", "ERROR:".red(), err);
            std::process::exit(1);
        }
        eprintln!();
        chosen_cmd = interactive_params(chosen_cmd, &inkfile_path, color);
    }

    match execute_command(chosen_cmd, &inkfile_path, opts.preview, color) {
        Ok(status) => {
            if let Some(code) = status.code() {
                std::process::exit(code)
            }
        }
        Err(err) => {
            eprintln!("{} {}", "ERROR:".red(), err);
            std::process::exit(1)
        }
    }
}

/// Prompt for missing parameters interactively.
fn interactive_params(mut chosen_cmd: Command, inkfile_path: &str, color: bool) -> Command {
    loop {
        let rv = KeyPrompt::with_theme(&ColoredTheme::default())
            .with_text(&format!("Execute step {}?", chosen_cmd.name))
            .items(&['y', 'n', 'p'])
            .default(0)
            .interact()
            .unwrap();
        if rv == 'y' {
            break;
        } else if rv == 'p' {
            match execute_command(chosen_cmd.clone(), inkfile_path, true, color) {
                Ok(_) => {
                    eprintln!(); // empty space
                    continue;
                }
                Err(err) => {
                    eprintln!("{} {}", "ERROR:".red(), err);
                    std::process::exit(1)
                }
            }
        } else {
            eprintln!("Skipping command {}", chosen_cmd.name);
            std::process::exit(0);
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
                .unwrap();
            arg.val = rv
        }
    }
    for flag in chosen_cmd.option_flags.iter_mut() {
        if !flag.takes_value {
            if flag.name == "verbose" {
                break;
            }
            if flag.val != "true" {
                let rv: bool = Confirmation::with_theme(&ColoredTheme::default())
                    .with_text(&format!("{}: Set {} option?", chosen_cmd.name, flag.name))
                    .default(false)
                    .interact()
                    .unwrap();
                if rv == true {
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
                    .unwrap();
                if is_invalid_number(flag.validate_as_number, &rv) {
                    log_expect_number(&name);
                    continue;
                } else {
                    break;
                };
            }
            flag.val = rv
        }
    }
    chosen_cmd
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
    inkfile_path: String,
}

// We must parse flags first to handle global flags and implicit defaults
fn pre_parse(mut args: Vec<String>) -> (CustomOpts, Vec<String>) {
    let mut opts = CustomOpts::default();
    let early_exit_modifiers = sset!["-h", "--help", "-V", "--version"];
    // Loop through all args and parse
    let mut inkfile_index = 1000;
    if args.len() == 1 {
        args.insert(1, "_default".to_string());
    }
    // If the first argument is a markdown file or '-' assume it is a inkfile arg
    // This allows us to use it as an interpretter without specifying '--inkfile'
    if args.len() > 1 && args[1] == "-" || args[1].ends_with(".md") {
        args.insert(1, "--inkfile".to_string());
    }
    for i in 1..args.len() {
        let arg = &args[i];
        if i == inkfile_index {
            opts.inkfile_path = canonical_path(arg);
        } else if arg == "-i" || arg == "--interactive" {
            opts.interactive = true;
        } else if arg == "--inkfile" {
            if let Some(idx) = arg.find('=') {
                opts.inkfile_path = canonical_path(&arg[(idx + 1)..]);
            } else {
                inkfile_index = i + 1
            }
        } else if arg == "--preview" || arg == "-p" {
            opts.preview = true;
        } else if !arg.starts_with('-') || early_exit_modifiers.contains(arg) {
            break; // no more parsing to do as a subcommand has been called
        } else if arg == "-" {
            continue; // stdin file input
        } else {
            // This may be a flag for the default command.
            args.insert(i, "_default".to_string());
            break;
        }
    }
    (opts, args)
}

// converts a given path str to a canonical path String
fn canonical_path(p: &str) -> String {
    Path::new(p).to_str().unwrap().to_string()
}

fn find_inkfile(inkfile_path: &str) -> (Result<String, String>, String) {
    let (inkfile, the_path) = inkjet::loader::read_inkfile(&inkfile_path);

    if inkfile.is_err() {
        if inkfile_path == "" || inkfile_path == "./inkjet.md" {
            // Just log a warning and let the process continue
            eprintln!("{} no inkjet.md found", "WARNING:".yellow());
        } else {
            // This is a custom inkfile. We exit with an error it's not found
            eprintln!(
                "{} specified inkfile \"{}\" not found",
                "ERROR:".red(),
                inkfile_path
            );
            std::process::exit(1);
        }
        (inkfile, "".to_string())
    } else {
        // Find the absolute path to the inkfile
        let absolute_path = fs::canonicalize(&the_path)
            .expect("canonicalize inkfile path failed")
            .to_str()
            .unwrap()
            .to_string();
        (inkfile, absolute_path)
    }
}

fn custom_inkfile_path_arg<'a, 'b>() -> Arg<'a, 'b> {
    // This is needed to prevent clap from complaining about the custom flag check
    // within find_inkfile(). It should be removed once clap 3.x is released.
    // See https://github.com/clap-rs/clap/issues/748
    Arg::with_name("inkfile")
        .help("Path to a different inkfile you want to use")
        .long("inkfile")
        .takes_value(true)
        .multiple(false)
}

fn build_subcommands<'a, 'b>(
    mut cli_app: App<'a, 'b>,
    color_setting: AppSettings,
    opts: &CustomOpts,
    subcommands: &'a [Command],
) -> App<'a, 'b> {
    for c in subcommands {
        let mut subcmd = SubCommand::with_name(&c.name)
            .about(c.desc.as_ref())
            .setting(color_setting)
            .setting(AppSettings::AllowNegativeNumbers);
        if !c.subcommands.is_empty() {
            subcmd = build_subcommands(subcmd, color_setting, opts, &c.subcommands);
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
        if c.alias != "" {
            subcmd = subcmd.visible_alias(c.alias.as_str());
        }
        cli_app = cli_app.subcommand(subcmd);
    }

    cli_app
}

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
                }
            }
        }
    }
    command
}

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
                .or(Some(""))
                .unwrap()
                .to_string();

            if is_invalid_number(flag.validate_as_number, &raw_value) {
                log_expect_number(&flag.name);
                std::process::exit(1);
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

fn is_invalid_number(is_num: bool, raw_value: &str) -> bool {
    if !is_num || raw_value == "" {
        return false;
    }
    // Try converting to an integer or float to validate it
    raw_value.parse::<isize>().is_err() && raw_value.parse::<f32>().is_err()
}

fn log_expect_number(name: &str) {
    eprintln!(
        "{} flag `{}` expects a numerical value",
        "ERROR:".red(),
        name
    );
}
