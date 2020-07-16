use colored::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{Error, ErrorKind, Result, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process;
use std::process::ExitStatus;

use clap::crate_name;

use crate::command::Command;

// takes a source string and generates a temporary hash for the filename.
fn hash_source(s: &str) -> String {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

// we append  `set -e` to these shells as a sensible default
fn needs_set_e(s: &str) -> bool {
    s == "sh" || s == "bash" || s == "" || s == "dash" || s == "zsh"
}

pub fn execute_command(
    mut cmd: Command,
    inkfile_path: &str,
    preview: bool,
    color: bool,
) -> Result<ExitStatus> {
    if cmd.script.source == "" {
        let msg = "Command has no script.";
        return Err(Error::new(ErrorKind::Other, msg));
    }

    if cmd.script.executor == "" && !cmd.script.source.trim().starts_with("#!") {
        cmd.script.executor = String::from("sh"); // default to default shell
    }

    if preview {
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
                if needs_set_e(&cmd.script.executor) {
                    let s = format!("set -e\n{}", cmd.script.source);
                    child_stdin.write_all(s.as_bytes())?;
                } else {
                    child_stdin.write_all(cmd.script.source.as_bytes())?;
                }
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
        let parent_dir = get_parent_dir(&inkfile_path);
        let mut tempfile = String::new();
        let mut child = prepare_command(&cmd, &parent_dir, &mut tempfile);
        child = add_utility_variables(child, inkfile_path);
        child = add_flag_variables(child, &cmd);
        let result = child
            .spawn()
            .unwrap_or_else(|err| {
                if tempfile != "" && std::fs::remove_file(&tempfile).is_err() {
                    eprintln!("{} Failed to delete file {}", "ERROR:".red(), tempfile);
                }
                eprintln!("{} {}", "ERROR:".red(), err);
                process::exit(1);
            })
            .wait();
        if tempfile != "" && std::fs::remove_file(&tempfile).is_err() {
            eprintln!("{} Failed to delete file {}", "ERROR:".red(), tempfile);
        }
        result
    }
}

fn prepare_command(cmd: &Command, parent_dir: &str, tempfile: &mut String) -> process::Command {
    let mut executor = cmd.script.executor.clone();
    let source = cmd.script.source.trim();
    if source.starts_with("#!") || executor == "go" {
        let hash = hash_source(source);
        // Handle Golang executor by default
        let data = if executor == "go" && !source.starts_with("#!") {
            String::from("#!/usr/bin/env yaegi\n") + source
        } else {
            String::from(source)
        };
        *tempfile = format!("{}/.order.{}", parent_dir, hash);
        std::fs::write(&tempfile, data)
            .unwrap_or_else(|_| panic!("Unable to write file {}", &tempfile));
        let meta = std::fs::metadata(&tempfile).expect("Unable to read file permissions");
        let mut perms = meta.permissions();
        perms.set_mode(0o775);
        std::fs::set_permissions(&tempfile, perms).expect("Could not set permissions");

        process::Command::new(tempfile)
    } else {
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
                child.arg("eval").arg("-T").arg(source);
                child
            }
            // If no language is specified, we use the default shell
            "" | "sh" | "bash" | "zsh" | "dash" => {
                if executor == "" {
                    executor = "sh".to_owned()
                }
                let mut child = process::Command::new(executor);
                let top = "set -e"; // a sane default for scripts
                let src = format!("{}\n{}", top, source);
                child.arg("-c").arg(src);
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
}

// Find the absolute path to the inkfile's parent directory
fn get_parent_dir(inkfile_path: &str) -> String {
    Path::new(&inkfile_path)
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
}

// Add some useful environment variables that scripts can use
fn add_utility_variables(mut child: process::Command, inkfile_path: &str) -> process::Command {
    // This allows us to call "$INKJET command" instead of "inkjet --inkfile <path> command"
    // inside scripts so that they can be location-agnostic (not care where they are
    // called from). This is useful for global inkfiles especially.
    child.env(
        "INKJET",
        format!("{} --inkfile {}", crate_name!(), inkfile_path),
    );
    // This allows us to refer to the directory the inkfile lives in which can be handy
    // for loading relative files to it.
    child.env("INKJET_DIR", get_parent_dir(inkfile_path));

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
        child.env(arg.name.replace("-", "_"), val);
    }

    // Add all optional flags as environment variables if they have a value
    for flag in &cmd.option_flags {
        if flag.val != "" {
            child.env(flag.name.replace("-", "_"), flag.val.clone());
        }
    }

    child
}
