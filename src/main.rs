use std::collections::HashSet;
use std::env;
use std::path::Path;

use clap::{crate_name, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand};
use colored::*;

use mask::command::Command;
use mask::executor::execute_command;

fn main() {
    let color = env::var_os("NO_COLOR").is_some();
    let color_setting = if color {
        AppSettings::ColoredHelp
    } else {
        AppSettings::ColorNever
    };
    let cli_app = App::new(crate_name!())
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::AllowNegativeNumbers)
        .setting(AppSettings::SubcommandRequired)
        .setting(color_setting)
        // TODO: respect NO_COLOR environment variable
        // TEMP: disable while debug
        // .setting(AppSettings::DisableHelpSubcommand)
        // .setting(AppSettings::DisableHelpFlags)
        .version(crate_version!())
        .arg(custom_maskfile_path_arg())
        .arg_from_usage("-p --print 'Print the command code and exit'");

    // let global_cli = App::new(crate_name!())
    //     .setting(AppSettings::DisableHelpSubcommand)
    //     .setting(AppSettings::DisableHelpFlags)
    //     .setting(AppSettings::AllowExternalSubcommands)
    //     .settings(AppSettings::)
    //     .setting(AppSettings::ColorNever)
    //     .arg(custom_maskfile_path_arg())
    //     .arg_from_usage("-i --interactive 'Execute each command in the document sequentially prompting for arguments'")
    //     .arg_from_usage("-p --print 'Print the command code and exit'");

    // // Initial parse for global flags
    // let global_matches = global_cli.get_matches_from_safe(env::args());
    // println!("{:?}", global_matches);
    let (opts, args) = pre_parse(env::args().collect());

    let maskfile = find_maskfile(&opts.maskfile_path);
    if maskfile.is_err() {
        // If the maskfile can't be found, at least parse for --version or --help
        cli_app.get_matches();
        return;
    }

    let root_command = mask::parser::build_command_structure(maskfile.unwrap());
    let matches = build_subcommands(cli_app, &root_command.subcommands).get_matches_from(args);
    let chosen_cmd = find_command(&matches, &root_command.subcommands)
        .expect("SubcommandRequired failed to work");

    match execute_command(chosen_cmd, opts.maskfile_path) {
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

#[derive(Default)]
struct CustomOpts {
    interactive: bool,
    print: bool,
    maskfile_path: String,
}

// We must parse flags first to handle global flags and implicit defaults
fn pre_parse(mut args: Vec<String>) -> (CustomOpts, Vec<String>) {
    let mut opts = CustomOpts::default();
    let early_exit_modifiers = sset!["-h", "--help", "-V", "--version"];
    // Loop through all args and parse
    let mut maskfile_arg_found = false;
    let mut maskfile_index = 1000;
    for i in 0..args.len() {
        let arg = &args[i];
        if i == maskfile_index {
            opts.maskfile_path = canonical_path(arg);
        } else if arg == "-i" || arg == "--interactive" {
            opts.interactive = true;
        } else if arg == "--maskfile" {
            maskfile_arg_found = true;
            if let Some(idx) = arg.find('=') {
                opts.maskfile_path = canonical_path(&arg[(idx + 1)..]);
            } else {
                maskfile_index = i + 1
            }
        } else if arg == "--print" {
            opts.print = true;
        } else if !arg.starts_with('-') || early_exit_modifiers.contains(arg) {
            break; // no more parsing to do as a subcommand has been called
        } else {
            // This may be a flag for the default command.
            args.insert(i, "_default".to_string());
            break;
        }
    }
    if !maskfile_arg_found {
        opts.maskfile_path = canonical_path("./maskfile.md");
    }
    (opts, args)
}

// converts a given path str to a canonical path String
fn canonical_path(p: &str) -> String {
    Path::new(p).to_str().unwrap().to_string()
}

fn find_maskfile(maskfile_path: &str) -> Result<String, String> {
    let maskfile = mask::loader::read_maskfile(&maskfile_path);

    if maskfile.is_err() {
        // Check if this is a custom maskfile
        if maskfile_path != "./maskfile.md" {
            // Exit with an error it's not found
            eprintln!("{} specified maskfile not found", "ERROR:".red());
            std::process::exit(1);
        } else {
            // Just log a warning and let the process continue
            println!("{} no maskfile.md found", "WARNING:".yellow());
        }
    }

    maskfile
}

fn custom_maskfile_path_arg<'a, 'b>() -> Arg<'a, 'b> {
    // This is needed to prevent clap from complaining about the custom flag check
    // within find_maskfile(). It should be removed once clap 3.x is released.
    // See https://github.com/clap-rs/clap/issues/748
    Arg::with_name("maskfile")
        .help("Path to a different maskfile you want to use")
        .long("maskfile")
        .takes_value(true)
        .multiple(false)
}

fn build_subcommands<'a, 'b>(mut cli_app: App<'a, 'b>, subcommands: &'a [Command]) -> App<'a, 'b> {
    for c in subcommands {
        let mut subcmd = SubCommand::with_name(&c.name)
            .about(c.desc.as_ref())
            .setting(AppSettings::ColoredHelp)
            .setting(AppSettings::AllowNegativeNumbers);
        if !c.subcommands.is_empty() {
            subcmd = build_subcommands(subcmd, &c.subcommands);
            // If this parent command has no script source, require a subcommand.
            if c.script.source == "" {
                subcmd = subcmd.setting(AppSettings::SubcommandRequired);
            }
        }

        // Add all required and optional arguments
        for a in &c.args {
            let arg = Arg::with_name(&a.name);
            subcmd = subcmd.arg(arg.required(a.required));
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
        arg.val = matches.value_of(arg.name.clone()).unwrap().to_string();
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

            if flag.validate_as_number && raw_value != "" {
                // Try converting to an integer or float to validate it
                if raw_value.parse::<isize>().is_err() && raw_value.parse::<f32>().is_err() {
                    eprintln!(
                        "{} flag `{}` expects a numerical value",
                        "ERROR:".red(),
                        flag.name
                    );
                    std::process::exit(1);
                }
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
