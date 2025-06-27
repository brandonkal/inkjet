// Copyright 2020 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

#[cfg(test)]
#[cfg(not(windows))]
mod interactive {
    use assert_cmd::{cargo, crate_name};
    use rexpect::error::Error;
    use rexpect::spawn;
    use std::env;

    fn cargo_bin() -> String {
        let path = cargo::cargo_bin(crate_name!());
        if path.is_file() {
            return path.to_string_lossy().to_string();
        }
        panic!("Could not locate cargo_bin {path:?}")
    }

    fn do_interactive() -> Result<(), Error> {
        unsafe { env::set_var("NO_COLOR", "1") };
        let exec = format!(
            "{} --inkfile tests/simple_case/inkjet.md -i echo",
            cargo_bin()
        );
        let mut p = spawn(&exec, Some(6_000))?;
        p.exp_string("Execute step echo?")?;
        p.send("y")?;
        p.flush()?;
        p.exp_string("Enter option for num")?;
        p.send_line("42")?;
        p.exp_string("Enter option for required *")?;
        p.send_line("this_was_required")?;
        p.exp_string("Enter option for any")?;
        p.send_line("any_value")?;
        p.exp_string("Enter value for name *")?;
        p.send_line("Brandon")?;
        p.exp_string("Enter value for optional (default)")?;
        p.send_line("")?;
        p.exp_string("Enter value for not_required")?;
        p.send_line("")?;
        p.exp_string("Enter value for extras")?;
        p.send_line("--extra1=1 --extra2 extra3")?;
        p.exp_string("Hello Brandon! Optional arg is \"default\". Number is \"42\". Required is \"this_was_required\". Any is \"any_value\". extras is \"--extra1=1 --extra2 extra3\"")?;
        Ok(())
    }

    fn do_interactive_preview() -> Result<(), Error> {
        let exec = format!(
            "{} --inkfile tests/simple_case/inkjet.md -i echo",
            cargo_bin()
        );
        let mut p = spawn(&exec, Some(6_000))?;
        p.exp_string("Execute step echo?")?;
        p.send("p")?;
        p.flush()?;
        p.exp_string("Optional arg is")?; // we look for a portion because of color
        Ok(())
    }

    fn do_interactive_skip() -> Result<(), Error> {
        let exec = format!(
            "{} --inkfile tests/simple_case/inkjet.md -i echo",
            cargo_bin()
        );
        let mut p = spawn(&exec, Some(6_000))?;
        p.exp_string("Execute step echo?")?;
        p.send("n")?;
        p.flush()?;
        match p.process.status() {
            Some(s) => match s {
                rexpect::process::wait::WaitStatus::Exited(_, code) => {
                    if code == 0 {
                        return Ok(());
                    }
                    panic!("process exited with code {code}");
                }
                _ => Ok(()),
            },
            _ => panic!("wait failed"),
        }
    }

    #[test]
    fn interactive() {
        do_interactive().unwrap_or_else(|e| panic!("inkjet job failed with {e}"));
    }

    #[test]
    fn interactive_preview() {
        do_interactive_preview().unwrap_or_else(|e| panic!("inkjet job failed with {e}"));
    }

    #[test]
    fn interactive_skip() {
        do_interactive_skip().unwrap_or_else(|e| panic!("inkjet job failed with {e}"));
    }
}
