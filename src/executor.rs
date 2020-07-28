#![warn(clippy::indexing_slicing)]
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

/// takes a source string and generates a temporary hash for the filename.
fn hash_source(s: &str) -> String {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// we append  `set -e` to these shells as a sensible default
fn needs_set_e(s: &str) -> bool {
    s == "sh" || s == "bash" || s == "" || s == "dash" || s == "zsh"
}

/// Executes a shell function that finds all inkjet.md files in a directory and
/// merges them together. Useful for projects with several inkjet.md files.
/// returns the output of the merge operation: a new inkfile content String
pub fn execute_merge_command(inkfile_path: &str) -> String {
    let parent_dir = get_parent_dir(inkfile_path);
    let convert_code = "for f in $(find \"$(pwd -P)\" -name inkjet.md | awk -F/ '{print NF-1 \" \" $0 }' | sort -n | cut -d ' ' -f 2-); do printf '<!-- inkfile: %s -->\n' \"$f\"; cat \"$f\"; done";
    let out = process::Command::new("sh")
        .arg("-c")
        .arg(convert_code)
        .current_dir(parent_dir)
        .output()
        .expect("Inkjet import command failed to start");
    if !out.status.success() {
        eprintln!("{} {}", "ERROR:".red(), "Inkjet import command failed");
        process::exit(1);
    }
    String::from_utf8(out.stdout).expect("Inkjet import command failed")
}
/// Execute a given command using its executor or sh. If preview is set, the script will be printed instead.
pub fn execute_command(
    mut cmd: Command,
    inkfile_path: &str,
    preview: bool,
    color: bool,
    fixed_dir: bool,
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
                let mut child_stdin = child.stdin.take().expect("unable to build stdin");
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
        let mut local_inkfile = cmd.inkjet_file.trim();
        if local_inkfile == "" {
            local_inkfile = inkfile_path
        }
        let parent_dir = get_parent_dir(local_inkfile);
        let mut tempfile = String::new();
        let mut child = prepare_command(&cmd, &parent_dir, &mut tempfile);
        child = add_utility_variables(child, inkfile_path, local_inkfile);
        child = add_flag_variables(child, &cmd);
        if fixed_dir {
            child.current_dir(parent_dir);
        }
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

/// `prepare_command` takes a Command struct and builds a `process::Command` that can then be executed as a child process.
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

/// Find the absolute path to the inkfile's parent directory
fn get_parent_dir(inkfile_path: &str) -> String {
    Path::new(&inkfile_path)
        .parent()
        .expect("unable to find parent path for inkfile")
        .to_str()
        .expect("inkfile parent path contains invalid UTF-8 characters")
        .to_owned()
}

/// Add some useful environment variables that scripts can use
fn add_utility_variables(
    mut child: process::Command,
    inkfile_path: &str,
    local_inkfile_path: &str,
) -> process::Command {
    // This allows us to call "$INKJET command" instead of "inkjet --inkfile <path> command"
    // inside scripts so that they can be location-agnostic (not care where they are
    // called from). This is useful for global inkfiles especially.
    // $INKJET always refers to the root inkjet script
    child.env(
        "INKJET",
        format!("{} --inkfile {}", crate_name!(), inkfile_path),
    );
    // $INK is shorthand for "$INKJET command". The difference here is that it resolves to the local inkjet.md which
    // could differ from $INKJET if the file was imported.
    child.env(
        "INK",
        format!("{} --inkfile {}", crate_name!(), local_inkfile_path),
    );
    // This allows us to refer to the directory the inkfile lives in which can be handy
    // for loading relative files to it.
    child.env("INKJET_DIR", get_parent_dir(inkfile_path));
    // This is the same as INKJET_DIR, but could differ for imported inkjet.md files.
    child.env("INK_DIR", get_parent_dir(local_inkfile_path));
    // Environment variable is set if this file was imported from another.
    if local_inkfile_path != inkfile_path {
        child.env("INKJET_IMPORTED", "true");
    }

    child
}

fn add_flag_variables(mut child: process::Command, cmd: &Command) -> process::Command {
    // Add all required args as environment variables
    for arg in &cmd.args {
        let val = if arg.val.is_empty() && arg.default.is_some() {
            arg.default
                .as_ref()
                .expect("unable to ref command default arg")
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
