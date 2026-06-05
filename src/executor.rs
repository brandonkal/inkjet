// Copyright 2020 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

use std::env;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process;

use crate::command::CommandBlock;
use crate::utils;

/// we append  `set -e` to these shells as a sensible default
fn needs_set_e(s: &str) -> bool {
    s == "sh" || s == "bash" || s.is_empty() || s == "dash" || s == "zsh"
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
        let parent_dir = get_parent_dir(inkfile_path);
        let (mut child, executor, tempfile) = prepare_command(&cmd);
        child = add_utility_variables(child, inkfile_path);
        child = add_flag_variables(child, &cmd);
        if fixed_dir {
            child.current_dir(parent_dir);
        }
        execute_prepared_command(child, executor, tempfile, color)
    }
}

#[cfg(unix)]
fn execute_prepared_command(
    mut child: process::Command,
    mut executor: String,
    tempfile: Option<tempfile::TempPath>,
    color: bool,
) -> Option<io::Result<process::ExitStatus>> {
    use std::os::unix::process::CommandExt;

    // On successful exec, this process is replaced and Rust destructors do not run.
    // This is intentional for better signal handling; shebang temp files may remain.
    let err = child.exec();
    if err.kind() == io::ErrorKind::NotFound {
        if executor.is_empty() {
            executor = String::from("the executor")
        }
        eprintln!(
            "{} Please check if {} is installed to run the command.",
            utils::error_msg(color),
            executor
        );
    }
    drop(tempfile);
    Some(io::Result::Err(err))
}

#[cfg(windows)]
fn execute_prepared_command(
    mut child: process::Command,
    mut executor: String,
    tempfile: Option<tempfile::TempPath>,
    color: bool,
) -> Option<io::Result<process::ExitStatus>> {
    use std::os::windows::process::CommandExt;

    // Keep the child attached to the same console/process group so console control
    // events such as Ctrl-C are delivered to both inkjet and the child process.
    child.creation_flags(0);
    let spawned_child = child.spawn();
    match spawned_child {
        Err(err) => {
            if err.kind() == io::ErrorKind::NotFound {
                if executor.is_empty() {
                    executor = String::from("the executor")
                }
                eprintln!(
                    "{} Please check if {} is installed to run the command.",
                    utils::error_msg(color),
                    executor
                );
            }
            drop(tempfile);
            Some(io::Result::Err(err))
        }
        Ok(mut child) => {
            let r = child.wait();
            drop(tempfile);
            Some(r)
        }
    }
}

/// `prepare_command` takes a CommandBlock struct and builds a `process::Command` that can then be executed as a child process.
fn prepare_command(cmd: &CommandBlock) -> (process::Command, String, Option<tempfile::TempPath>) {
    let mut executor = cmd.script.executor.clone();
    let source = cmd.script.source.trim();
    if source.starts_with("#!") {
        let mut tempfile = tempfile::Builder::new()
            .prefix("inkjet-order-")
            .tempfile()
            .expect("Inkjet: Unable to create temporary file");
        tempfile
            .write_all(source.as_bytes())
            .expect("Inkjet: Unable to write temporary file");

        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tempfile
                .as_file()
                .metadata()
                .expect("Inkjet: Unable to read file permissions")
                .permissions();
            perms.set_mode(0o775);
            tempfile
                .as_file()
                .set_permissions(perms)
                .expect("Inkjet: Could not set permissions");
        }

        let temp_path = tempfile.into_temp_path();
        let child = process::Command::new(&temp_path);
        (child, String::from("the executor"), Some(temp_path))
    } else {
        match executor.as_ref() {
            "js" | "javascript" => {
                let mut child;
                child = process::Command::new("node");
                child.arg("-e").arg(source);
                (child, String::from("node"), None)
            }
            "py" | "python" | "python3" => {
                #[cfg(not(windows))]
                let the_executor = "python3";

                #[cfg(windows)]
                let the_executor = "python";

                let mut child = process::Command::new(the_executor);
                child.arg("-c").arg(source);
                (child, String::from(the_executor), None)
            }
            "rb" | "ruby" => {
                let mut child = process::Command::new("ruby");
                child.arg("-e").arg(source);
                (child, String::from("ruby"), None)
            }
            "php" => {
                let mut child = process::Command::new("php");
                child.arg("-r").arg(source);
                (child, String::from("php"), None)
            }
            "ts" | "typescript" => {
                let mut child = process::Command::new("deno");
                child.arg("eval").arg("--ext=ts").arg(source);
                (child, String::from("deno"), None)
            }
            "go" => {
                let mut child = process::Command::new("yaegi");
                child.arg("-e").arg(source);
                (child, String::from("yaegi"), None)
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
                (child, executor, None)
            }
            #[cfg(windows)]
            "cmd" | "batch" => {
                let mut child = process::Command::new("cmd.exe");
                child.arg("/c").arg(source);
                (child, "cmd.exe".to_string(), None)
            }
            #[cfg(windows)]
            "powershell" => {
                let mut child = process::Command::new("powershell.exe");
                child.arg("-c").arg(source);
                (child, "powershell.exe".to_string(), None)
            }
            // Any other executor that supports -c (fish, etc...)
            _ => {
                let mut child = process::Command::new(&executor); // cov:ignore
                child.arg("-c").arg(source); // cov:ignore
                (child, executor, None) // cov:ignore
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
fn add_utility_variables(mut child: process::Command, inkfile_path: &str) -> process::Command {
    let exe_path = match env::current_exe() {
        Ok(path) => path.to_string_lossy().into_owned(),
        _ => "inkjet".to_owned(),
    };
    // This allows us to call "$INK command" instead of "inkjet --inkfile <path> command"
    // inside scripts so that they can be location-agnostic (not care where they are
    // called from). This is useful for global inkfiles especially.
    child.env("INK", format!("{exe_path} --inkfile {inkfile_path}"));
    // This allows us to refer to the directory the inkfile lives in which can be handy
    // for loading relative files to it.
    child.env("INK_DIR", get_parent_dir(inkfile_path));

    child
}

fn add_flag_variables(mut child: process::Command, cmd: &CommandBlock) -> process::Command {
    // Add all required args as environment variables
    for arg in &cmd.args {
        let val = if arg.val.is_empty() {
            arg.default.as_deref().unwrap_or("")
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
