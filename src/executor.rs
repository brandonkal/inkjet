use colored::*;
use std::io::{Error, ErrorKind, Result, Write};
use std::path::Path;
use std::process;
use std::process::ExitStatus;

use clap::crate_name;

use crate::command::Command;

pub fn execute_command(
    cmd: Command,
    maskfile_path: &str,
    print: bool,
    color: bool,
) -> Result<ExitStatus> {
    if cmd.script.source == "" {
        let msg = "Command has no script.";
        return Err(Error::new(ErrorKind::Other, msg));
    }

    if cmd.script.executor == "" {
        let msg = "Command script requires a lang code which determines which executor to use.";
        return Err(Error::new(ErrorKind::Other, msg));
    }

    if print {
        if !color {
            print!("{}", cmd.script.source);
            process::exit(0);
        }
        let mut bat_cmd = match process::Command::new("bat")
            .args(&["--plain", "--language", &cmd.script.executor])
            .stdin(process::Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                let mut child_stdin = child.stdin.take().unwrap();
                child_stdin.write_all(cmd.script.source.as_bytes())?;
                child
            }
            Err(e) => {
                if ErrorKind::NotFound == e.kind() {
                    print!("{}", cmd.script.source);
                    process::exit(0);
                }
                eprintln!("{} {}", "ERROR:".red(), e);
                process::exit(1);
            }
        };
        bat_cmd.wait()
    } else {
        let mut child = prepare_command(&cmd);
        child = add_utility_variables(child, maskfile_path);
        child = add_flag_variables(child, &cmd);
        child.spawn()?.wait()
    }
}

fn prepare_command(cmd: &Command) -> process::Command {
    let executor = cmd.script.executor.clone();
    let source = cmd.script.source.clone();

    match executor.as_ref() {
        "js" | "javascript" => {
            let mut child;
            child = process::Command::new("node");
            child.arg("-e").arg(source);
            child
        }
        "py" | "python" => {
            let mut child = process::Command::new("python");
            child.arg("-c").arg(source);
            child
        }
        "rb" | "ruby" => {
            let mut child = process::Command::new("ruby");
            child.arg("-e").arg(source);
            child
        }
        "php" => {
            let mut child = process::Command::new("php");
            child.arg("-r").arg(source);
            child
        }
        "ts" | "typescript" => {
            let mut child = process::Command::new("deno");
            // TODO: handle write to disk and call
            child.arg("run").arg(source);
            child
        }
        // Any other executor that supports -c (sh, bash, zsh, fish, dash, etc...)
        _ => {
            let mut child = process::Command::new(executor);
            child.arg("-c").arg(source);
            child
        }
    }
}

// Add some useful environment variables that scripts can use
fn add_utility_variables(mut child: process::Command, maskfile_path: &str) -> process::Command {
    // Find the absolute path to the maskfile's parent directory
    let parent_dir = Path::new(&maskfile_path)
        .parent()
        .unwrap()
        .to_str()
        .unwrap();

    // This allows us to call "$MASK command" instead of "mask --maskfile <path> command"
    // inside scripts so that they can be location-agnostic (not care where they are
    // called from). This is useful for global maskfiles especially.
    child.env(
        "MASK",
        format!("{} --maskfile {}", crate_name!(), maskfile_path),
    );
    // This allows us to refer to the directory the maskfile lives in which can be handy
    // for loading relative files to it.
    child.env("MASKFILE_DIR", parent_dir);

    child
}

fn add_flag_variables(mut child: process::Command, cmd: &Command) -> process::Command {
    // Add all required args as environment variables
    for arg in &cmd.args {
        let val = if arg.val.is_empty() && arg.default.is_some() {
            arg.default.as_ref().unwrap()
        } else {
            arg.val.as_str()
        };
        child.env(arg.name.clone(), val);
    }

    // Add all optional flags as environment variables if they have a value
    for flag in &cmd.option_flags {
        if flag.val != "" {
            child.env(flag.name.clone(), flag.val.clone());
        }
    }

    child
}
