// Copyright 2020 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::{env, fs};
use walkdir::WalkDir;

use crate::command::CommandBlock;
use crate::utils;

/// takes a source string and generates a temporary hash for the filename.
fn hash_source(s: &str) -> String {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// we append  `set -e` to these shells as a sensible default
fn needs_set_e(s: &str) -> bool {
    s == "sh" || s == "bash" || s.is_empty() || s == "dash" || s == "zsh"
}

/// Executes a shell function that finds all inkjet.md files in a directory and
/// merges them together. Useful for projects with several inkjet.md files.
/// returns the output of the merge operation: a new inkfile content String
pub fn execute_merge_command(inkfile_path: &str) -> Result<String, String> {
    let parent_dir = get_parent_dir(inkfile_path);
    // Collect paths that match the criteria
    let mut inkjet_files: Vec<PathBuf> = vec![];

    // Traverse the directory and find matching files
    for entry in WalkDir::new(parent_dir) {
        match entry {
            Ok(path) => {
                let filename = path.file_name().to_string_lossy();

                if filename == "inkjet.md" || filename.ends_with(".inkjet.md") {
                    inkjet_files.push(path.into_path());
                }
            }
            _ => continue,
        }
    }

    // Sort files by the number of directories in their path
    inkjet_files.sort_by_key(|path| path.components().count());

    // Prepare a String to collect the combined text
    let mut combined_text = String::new();

    // Append the content of each file with the required format
    for file in inkjet_files {
        let file_str = file.to_string_lossy();
        combined_text.push_str(&format!("<!-- inkfile: {file_str} -->\n"));

        match fs::read_to_string(&file) {
            Ok(content) => combined_text.push_str(&content),
            Err(e) => return Err(format!("Error reading file {file_str}: {e}")),
        }
    }

    Ok(combined_text)
}

fn run_bat(source: String, lang: &str) -> io::Result<process::Child> {
    match process::Command::new("bat")
        .args(["--plain", "--language", lang])
        .stdin(process::Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            let mut child_stdin = child
                .stdin
                .take()
                .expect("Inkjet (bat): unable to build stdin");
            child_stdin.write_all(source.as_bytes())?;
            io::Result::Ok(child)
        }
        Err(err) => io::Result::Err(err), // cov:include
    }
}

/// Execute a given command using its executor or sh. If preview is set, the script will be printed instead.
pub fn execute_command(
    mut cmd: CommandBlock,
    inkfile_path: &str,
    preview: bool,
    color: bool,
    fixed_dir: bool,
) -> Option<io::Result<process::ExitStatus>> {
    if cmd.script.source.is_empty() {
        let msg = "CommandBlock has no script."; // cov:include (unusual)
        return Some(Err(io::Error::other(msg))); // cov:include
    }

    if cmd.script.executor.is_empty() && !cmd.script.source.trim().starts_with("#!") {
        cmd.script.executor = String::from("sh"); // default to default shell
    }
    let source = if needs_set_e(&cmd.script.executor) {
        format!("set -e\n{}", &cmd.script.source)
    } else {
        cmd.script.source.clone()
    };

    if preview {
        if !color {
            print!("{source}");
            return None;
        }
        match run_bat(source.clone(), &cmd.script.executor) {
            Ok(mut child) => Some(child.wait()),
            Err(_) => {
                print!("{source}"); // cov:include (bat exists)
                None // cov:include
            }
        }
    } else {
        let mut local_inkfile = cmd.inkjet_file.trim();
        if local_inkfile.is_empty() {
            local_inkfile = inkfile_path
        }
        let parent_dir = get_parent_dir(local_inkfile);
        let mut tempfile = String::new();
        let (mut child, mut executor) = prepare_command(&cmd, &parent_dir, &mut tempfile);
        child = add_utility_variables(child, inkfile_path, local_inkfile);
        child = add_flag_variables(child, &cmd);
        if fixed_dir {
            child.current_dir(parent_dir);
        }
        let spawned_child = child.spawn();
        match spawned_child {
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    if executor.is_empty() {
                        executor = String::from("the executor")
                    }
                    eprintln!(
                        "{} Please check if {} is installed to run the command.",
                        utils::ERROR_MSG,
                        executor
                    );
                }
                delete_file(&tempfile); // cov:include (unusual)
                Some(io::Result::Err(err)) // cov:include
            }
            Ok(mut child) => {
                let r = child.wait();
                delete_file(&tempfile);
                Some(r)
            }
        }
    }
}

fn delete_file(file: &str) {
    if !file.is_empty() && std::fs::remove_file(file).is_err() {
        eprintln!(
            "{} Failed to delete temporary file {}",
            utils::ERROR_MSG,
            file
        ); // cov:ignore (unusual)
    }
}

/// `prepare_command` takes a CommandBlock struct and builds a `process::Command` that can then be executed as a child process.
fn prepare_command(
    cmd: &CommandBlock,
    parent_dir: &str,
    tempfile: &mut String,
) -> (process::Command, String) {
    let mut executor = cmd.script.executor.clone();
    let source = cmd.script.source.trim();
    if source.starts_with("#!") {
        let hash = hash_source(source);
        *tempfile = format!("{parent_dir}/.inkjet-order.{hash}");
        std::fs::write(&tempfile, source)
            .unwrap_or_else(|_| panic!("Inkjet: Unable to write file {}", &tempfile));

        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            let meta =
                std::fs::metadata(&tempfile).expect("Inkjet: Unable to read file permissions");
            let mut perms = meta.permissions();
            perms.set_mode(0o775);
            std::fs::set_permissions(&tempfile, perms).expect("Inkjet: Could not set permissions");
        }

        (
            process::Command::new(tempfile),
            String::from("the executor"),
        )
    } else {
        match executor.as_ref() {
            "js" | "javascript" => {
                let mut child;
                child = process::Command::new("node");
                child.arg("-e").arg(source);
                (child, String::from("node"))
            }
            "py" | "python" | "python3" => {
                #[cfg(not(windows))]
                let the_executor = "python3";

                #[cfg(windows)]
                let the_executor = "python";

                let mut child = process::Command::new(the_executor);
                child.arg("-c").arg(source);
                (child, String::from(the_executor))
            }
            "rb" | "ruby" => {
                let mut child = process::Command::new("ruby");
                child.arg("-e").arg(source);
                (child, String::from("ruby"))
            }
            "php" => {
                let mut child = process::Command::new("php");
                child.arg("-r").arg(source);
                (child, String::from("php"))
            }
            "ts" | "typescript" => {
                let mut child = process::Command::new("deno");
                child.arg("eval").arg("--ext=ts").arg(source);
                (child, String::from("deno"))
            }
            "go" => {
                let mut child = process::Command::new("yaegi");
                child.arg("-e").arg(source);
                (child, String::from("yaegi"))
            }
            // If no language is specified, we use the default shell
            "" | "sh" | "bash" | "zsh" | "dash" => {
                if executor.is_empty() {
                    executor = "sh".to_string() // cov:ignore (already added by execute_command)
                }
                let mut child = process::Command::new(&executor);
                let top = "set -e"; // a sane default for scripts
                let src = format!("{top}\n{source}");
                child.arg("-c").arg(src);
                (child, executor)
            }
            #[cfg(windows)]
            "cmd" | "batch" => {
                let mut child = process::Command::new("cmd.exe");
                child.arg("/c").arg(source);
                (child, "cmd.exe".to_string())
            }
            #[cfg(windows)]
            "powershell" => {
                let mut child = process::Command::new("powershell.exe");
                child.arg("-c").arg(source);
                (child, "powershell.exe".to_string())
            }
            // Any other executor that supports -c (fish, etc...)
            _ => {
                let mut child = process::Command::new(&executor); // cov:ignore
                child.arg("-c").arg(source); // cov:ignore
                (child, executor) // cov:ignore
            }
        }
    }
}

/// Find the absolute path to the inkfile's parent directory
fn get_parent_dir(inkfile_path: &str) -> String {
    Path::new(&inkfile_path)
        .parent()
        .expect("Inkjet: unable to find parent path for inkfile")
        .to_str()
        .expect("Inkjet: inkfile parent path contains invalid UTF-8 characters")
        .to_string()
}

/// Add some useful environment variables that scripts can use
fn add_utility_variables(
    mut child: process::Command,
    inkfile_path: &str,
    local_inkfile_path: &str,
) -> process::Command {
    let exe_path = match env::current_exe() {
        Ok(path) => path.to_string_lossy().into_owned(),
        _ => "inkjet".to_owned(),
    };
    // This allows us to call "$INKJET command" instead of "inkjet --inkfile <path> command"
    // inside scripts so that they can be location-agnostic (not care where they are
    // called from). This is useful for global inkfiles especially.
    // $INKJET always refers to the root inkjet script
    child.env("INKJET", format!("{exe_path} --inkfile {inkfile_path}"));
    // $INK is shorthand for "$INKJET command". The difference here is that it resolves to the local inkjet.md which
    // could differ from $INKJET if the file was imported.
    child.env("INK", format!("{exe_path} --inkfile {local_inkfile_path}"));
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

fn add_flag_variables(mut child: process::Command, cmd: &CommandBlock) -> process::Command {
    // Add all required args as environment variables
    for arg in &cmd.args {
        let val = if arg.val.is_empty() && arg.default.is_some() {
            arg.default // cov:include (tested by default_args integration)
                .as_ref()
                .expect("Inkjet: unable to ref command default arg")
        } else {
            arg.val.as_str()
        };
        child.env(arg.name.replace("-", "_"), val);
    }

    // Add all named flags as environment variables if they have a value
    for flag in &cmd.named_flags {
        if !flag.val.is_empty() {
            child.env(flag.name.replace("-", "_"), flag.val.clone());
        }
    }

    child
}
