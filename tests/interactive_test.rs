mod common;
pub use common::*;
use rexpect::errors::*;
use rexpect::spawn;

fn do_interactive() -> Result<()> {
    let exec = format!(
        "{} --inkfile tests/simple_case/inkjet.md -i echo",
        cargo_bin()
    );
    let mut p = spawn(&exec, Some(6_000))?;
    p.exp_string("Execute step echo?")?;
    p.send("y")?;
    p.flush()?;
    p.exp_string("Enter value for name")?;
    p.send_line("Brandon")?;
    p.exp_string("Enter value for optional")?;
    p.send_line("")?;
    p.exp_string("Enter option for num")?;
    p.send_line("42")?;
    p.exp_string("Hello Brandon! Optional arg is default. Number is 42")?;
    Ok(())
}

fn do_interactive_preview() -> Result<()> {
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

#[test]
fn interactive() {
    do_interactive().unwrap_or_else(|e| panic!("inkjet job failed with {}", e));
}

#[test]
fn interactive_preview() {
    do_interactive_preview().unwrap_or_else(|e| panic!("inkjet job failed with {}", e));
}
