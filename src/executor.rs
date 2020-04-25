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

pub fn execute_command(
    cmd: Command,
    maskfile_path: &str,
    preview: bool,
    color: bool,
) -> Result<ExitStatus> {
    if cmd.script.source == "" {
        let msg = "Command has no script.";
        return Err(Error::new(ErrorKind::Other, msg));
    }

    if cmd.script.executor == "" && !cmd.script.source.trim().starts_with("#!") {
        let msg = "Command script requires a language code or shebang which determines which executor to use.";
        return Err(Error::new(ErrorKind::Other, msg));
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
        // run_and_exit(bat_cmd);
        bat_cmd.wait()
    } else {
        let parent_dir = get_parent_dir(&maskfile_path);
        println!("Starting prepare_command");
        let mut tempfile = String::new();
        let mut child = prepare_command(&cmd, &parent_dir, &mut tempfile);
        child = add_utility_variables(child, maskfile_path);
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
        println!("Temp file after run {}", &tempfile);
        if tempfile != "" && std::fs::remove_file(&tempfile).is_err() {
            eprintln!("{} Failed to delete file {}", "ERROR:".red(), tempfile);
        }
        result
    }
}

fn prepare_command(cmd: &Command, parent_dir: &str, tempfile: &mut String) -> process::Command {
    let executor = cmd.script.executor.clone();
    let source = cmd.script.source.trim();
    // TODO: check if source starts with shebang magic num
    if source.starts_with("#!") || executor == "go" {
        println!("Script has shebang");
        let hash = hash_source(source);
        // Handle Golang executor by default
        let data = if executor == "go" && !source.starts_with("#!") {
            String::from("#!/usr/bin/env yaegi\n") + source
        } else {
            String::from(source)
        };
        *tempfile = format!("{}/.order.{}", parent_dir, hash);
        println!("{}", &tempfile);
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
                // TODO: handle write to disk and call
                child.arg("eval").arg(source);
                child
            }
            // Any other executor that supports -c (sh, bash, zsh, fish, dash, etc...)
            "" => {
                let mut child = process::Command::new("sh");
                child.arg("-c").arg(source);
                child
            }
            _ => {
                let mut child = process::Command::new(executor);
                child.arg("-c").arg(source);
                child
            }
        }
    }
}

// Find the absolute path to the maskfile's parent directory
fn get_parent_dir(maskfile_path: &str) -> String {
    Path::new(&maskfile_path)
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
}

// Add some useful environment variables that scripts can use
fn add_utility_variables(mut child: process::Command, maskfile_path: &str) -> process::Command {
    // This allows us to call "$MASK command" instead of "mask --maskfile <path> command"
    // inside scripts so that they can be location-agnostic (not care where they are
    // called from). This is useful for global maskfiles especially.
    child.env(
        "MASK",
        format!("{} --maskfile {}", crate_name!(), maskfile_path),
    );
    // This allows us to refer to the directory the maskfile lives in which can be handy
    // for loading relative files to it.
    child.env("MASKFILE_DIR", get_parent_dir(maskfile_path));

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
