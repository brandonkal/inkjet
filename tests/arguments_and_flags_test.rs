// Copyright 2020 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

use assert_cmd::prelude::*;
use clap::{crate_name, crate_version};
use predicates::str::contains;

mod common;
pub use common::InkjetCommandExt;
pub use common::*;

#[test]
fn positional_arguments() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## test (file) (test_case)

> Run tests

~~~bash
echo "Testing $test_case in $file"
~~~
"#,
    );

    common::run_inkjet(&inkfile_path)
        .command("test")
        .arg("the_file")
        .arg("some_test_case")
        .assert()
        .stdout(contains("Testing some_test_case in the_file"))
        .success();

    common::run_inkjet(&inkfile_path)
        .command("test")
        .arg("some_test_case")
        .assert()
        .stderr(contains(
            "error: The following required arguments were not provided:
    <test_case>",
        ))
        .failure();
}

#[test]
fn named_flags() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## serve

> Serve this directory

<!-- You must define OPTIONS right before your list of flags -->
**OPTIONS**
* port
    * flag: -p --port
    * type: string
    * desc: Which port to serve on

```bash
# Set a fallback port
PORT=${port:-8080}

if [[ "$verbose" == "true" ]]; then
    echo "Starting an http server on PORT: $PORT"
else
    echo $PORT
fi
```
"#,
    );

    common::run_inkjet(&inkfile_path)
        .command("serve")
        .arg("--port")
        .arg("1234")
        .assert()
        .stdout(contains("1234"))
        .success();

    // verbose is always available
    common::run_inkjet(&inkfile_path)
        .command("serve")
        .arg("--port")
        .arg("1234")
        .arg("--verbose")
        .assert()
        .stdout(contains("Starting an http server on PORT: 1234"))
        .success();
}

mod when_entering_negative_numbers {
    use super::*;

    #[test]
    fn allows_entering_negative_numbers_as_values() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## add (a) (b)
~~~bash
echo $(($a + $b))
~~~
"#,
        );

        common::run_inkjet(&inkfile_path)
            .cli("add -1 -3")
            .assert()
            .stdout(contains("-4"))
            .success();
    }

    #[test]
    fn allows_entering_negative_numbers_as_flag_values() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## add

**OPTIONS**
* a
    * flag: --a
    * type: string
* b
    * flag: --b
    * type: string

~~~bash
echo $(($a + $b))
~~~
"#,
        );

        common::run_inkjet(&inkfile_path)
            .cli("add --a -33 --b 17")
            .assert()
            .stdout(contains("-16"))
            .success();
    }
}

mod numerical_option_flag {
    use super::*;

    #[test]
    fn properly_validates_flag_with_type_number() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## integer

**OPTIONS**
* val
    * flag: --val
    * type: number

~~~bash
echo "Value: $val"
~~~
"#,
        );

        common::run_inkjet(&inkfile_path)
            .cli("integer --val 1111112222")
            .assert()
            .stdout(contains("Value: 1111112222"))
            .success();
    }

    #[test]
    fn properly_validates_negative_numbers() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## negative

**OPTIONS**
* val
    * flag: --val
    * type: number

~~~bash
echo "Value: $val"
~~~
"#,
        );

        common::run_inkjet(&inkfile_path)
            .cli("negative --val -123")
            .assert()
            .stdout(contains("Value: -123"))
            .success();
    }

    #[test]
    fn properly_validates_decimal_numbers() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## decimal

**OPTIONS**
* val
    * flag: --val
    * type: number

~~~bash
echo "Value: $val"
~~~
"#,
        );

        common::run_inkjet(&inkfile_path)
            .cli("decimal --val 123.3456789")
            .assert()
            .stdout(contains("Value: 123.3456789"))
            .success();
    }

    #[test]
    fn errors_when_value_is_not_a_number() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## notanumber

**OPTIONS**
* val
    * flag: --val
    * type: number

~~~bash
echo "This shouldn't render $val"
~~~
"#,
        );

        common::run_inkjet(&inkfile_path)
            .cli("notanumber --val a234")
            .assert()
            .stderr(contains("flag `val` expects a numerical value"))
            .failure();
    }

    #[test]
    fn ignores_the_option_if_not_supplied() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## nooption

**OPTIONS**
* val
    * flag: --val
    * type: number

~~~bash
echo "No arg this time"
~~~
"#,
        );

        common::run_inkjet(&inkfile_path)
            .cli("nooption")
            .assert()
            .stdout(contains("No arg this time"))
            .success();
    }
}

mod version_flag {
    use super::*;

    #[test]
    fn shows_the_correct_version_for_the_root_command() {
        let (_temp, inkfile_path) = common::inkfile("## foo");

        common::run_inkjet(&inkfile_path)
            .command("--version")
            .assert()
            .stdout(contains(format!("{} {}", crate_name!(), crate_version!())))
            .success();
    }

    #[test]
    fn exits_with_error_when_subcommand_has_version_flag() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## foo
```
echo "boo"
```
"#,
        );
        // The setting "VersionlessSubcommands" removes the version flags (-V, --version)
        // from subcommands. Only the root command has a version flag.

        common::run_inkjet(&inkfile_path)
            .command("foo")
            .arg("--version")
            .assert()
            .stderr(contains(
                "error: Found argument '--version' which wasn't expected, or isn't valid in this context",
            ))
            .failure();
    }
}
