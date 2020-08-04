use std::path::PathBuf;

use assert_cmd::prelude::*;
use clap::{crate_name, crate_version};
use predicates::str::contains;

mod common;
use common::InkjetCommandExt;

#[test]
fn help_has_usage() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## foo

<!-- a few more details -->
"#,
    );

    common::run_inkjet(&inkfile_path)
        .arg("--help")
        .assert()
        .stdout(contains("USAGE:"))
        .success();
}

#[test]
fn fails_on_bad_flag_type() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## check
OPTIONS
- flags: -b |invalid| An invalid type
```
echo "Should not print $b"
```
"#,
    );
    common::run_inkjet(&inkfile_path)
        .arg("check")
        .assert()
        .stderr(contains(
            "Invalid flag type 'invalid' Expected string | number | bool.",
        ))
        .failure();
}

#[test]
fn fails_on_invalid_number() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## check
OPTIONS
- flags: --num |number| A number
```
echo "Should not print $num"
```
"#,
    );
    common::run_inkjet(&inkfile_path)
        .arg("check")
        .arg("--num")
        .arg("abc")
        .assert()
        .stderr(contains("expects a numerical value"))
        .failure();
}

#[test]
fn simple_case_does_not_panic() {
    // This also checks to ensure no extra output is printed (i.e. debug println)
    common::run_binary()
        .current_dir("tests/simple_case")
        .assert()
        .stdout("expected output\n")
        .success();
}

#[test]
fn merge() {
    let part1 = r#"> This is the main info

# main

## echo

```
echo "Hello"
```"#;
    let part2 = r#"# second

## list

```
echo "Should not run as this is replaced"
```

## list

```
ls -1 ls-test
```"#;
    common::run_binary()
        .current_dir("tests/merge")
        .arg("--inkjet-print-all")
        .assert()
        .stdout(contains(part1))
        .stdout(contains(part2))
        .success();
    common::run_binary()
        .current_dir("tests/merge")
        .cli("second list")
        .assert()
        .stdout(contains("1\n2\n3"))
        .success();
}

// Using current_dir(".github") to make sure the default inkjet.md can't be found
mod when_no_inkfile_found_in_current_directory {
    use super::*;

    #[test]
    fn logs_warning_about_missing_inkfile() {
        common::run_inkjet(&PathBuf::from("./inkjet.md"))
            .current_dir(".github")
            .command("-V")
            .assert()
            .stderr(contains("no inkjet.md found"));
    }

    #[test]
    fn exits_without_error_for_help() {
        common::run_inkjet(&PathBuf::from("./inkjet.md"))
            .current_dir(".github")
            .command("--help")
            .assert()
            .stdout(contains("USAGE:"))
            .success();
    }

    #[test]
    fn exits_without_error_for_version() {
        common::run_inkjet(&PathBuf::from("./inkjet.md"))
            .current_dir(".github")
            .command("--version")
            .assert()
            .stdout(contains(format!("{} {}", crate_name!(), crate_version!())))
            .success();
    }

    #[test]
    fn exits_with_error_for_any_other_command() {
        common::run_inkjet(&PathBuf::from("./nothing.inkjet.md"))
            .current_dir("tests")
            .command("nothing")
            .assert()
            .code(1)
            .stderr(contains("error: Found argument 'nothing' which wasn't expected, or isn't valid in this context"))
            .failure();
    }
}

mod when_custom_specified_inkfile_not_found {
    use super::*;

    #[test]
    fn exits_with_error_for_help() {
        common::run_inkjet(&PathBuf::from("./nonexistent.md"))
            .command("--help")
            .assert()
            .code(10)
            .stderr(contains("specified inkfile \"./nonexistent.md\" not found"))
            .failure();
    }

    #[test]
    fn exits_with_error_for_version() {
        common::run_inkjet(&PathBuf::from("./nonexistent.md"))
            .command("--version")
            .assert()
            .code(10)
            .stderr(contains("specified inkfile \"./nonexistent.md\" not found"))
            .failure();
    }

    #[test]
    fn exits_with_error_for_any_other_command() {
        common::run_inkjet(&PathBuf::from("./nonexistent.md"))
            .command("what")
            .assert()
            .code(10)
            .stderr(contains("specified inkfile \"./nonexistent.md\" not found"))
            .failure();
    }
}

mod builds_command_tree {
    use super::*;

    #[test]
    fn works_with_second_h1() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
#!/usr/bin/env inkjet

> This is the main info

# main

## echo

```
echo "Hello"
```

# second

## echo

```
echo "Second"
```
"#,
        );
        common::run_inkjet(&inkfile_path)
            .command("--help")
            .assert()
            .stdout(contains("echo"))
            .stdout(contains("second"))
            .success();
    }
}

mod exits_with_the_child_process_status_code {
    use super::*;

    #[test]
    fn exits_with_success() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## success

~~~sh
exit 0
~~~
"#,
        );

        common::run_inkjet(&inkfile_path)
            .command("success")
            .assert()
            .code(0)
            .success();
    }

    #[test]
    fn exits_with_error1() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## failure

~~~sh
exit 1
~~~
"#,
        );

        common::run_inkjet(&inkfile_path)
            .command("failure")
            .assert()
            .code(1)
            .failure();
    }

    #[test]
    fn exits_with_error2() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## failure

~~~sh
exit 2
~~~
"#,
        );

        common::run_inkjet(&inkfile_path)
            .command("failure")
            .assert()
            .code(2)
            .failure();
    }
}
