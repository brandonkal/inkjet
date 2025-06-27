// Copyright 2020 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

use assert_cmd::prelude::*;
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

~~~powershell
param (
    $test_case = $env:test_case,
    $file = $env:file
)

Write-Output "Testing $test_case in $file"
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
            "error: the following required arguments were not provided:
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

mod choices {
    use super::*;
    use inkjet::runner;

    #[test]
    fn properly_validates_flag_with_choices() {
        let contents = r#"
## color

**OPTIONS**
* val
    * flag: --val
    * type: string
    * choices: RED, BLUE, GREEN

```bash
echo "Value: $val"
```

```powershell
param (
    $in = $env:val
)
Write-Output "Value: $in"
```
"#;
        let (_temp, inkfile_path) = common::inkfile(contents);

        /// Creates vector of strings, Vec<String>
        macro_rules! svec {
            ($($x:expr),*) => (vec![$($x.to_string()),*]);
        }

        let (rc, _, _) = runner::run(
            svec!("inkjet", "--inkfile", contents, "color", "--val", "RED"),
            false,
        );
        assert_eq!(0, rc);

        common::run_inkjet(&inkfile_path)
            .cli("color --val RED")
            .assert()
            .stdout(contains("Value: RED"))
            .success();
    }

    #[test]
    fn out_of_choices() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## color

**OPTIONS**
* val
    * flag: --val
    * type: string
    * choices: RED, BLUE, GREEN

```bash
echo "Value: $val"
```

```powershell
param (
    $in = $env:val
)
Write-Output "Value: $in"
```
"#,
        );

        common::run_inkjet(&inkfile_path)
            .cli("color --val YELLOW")
            .assert()
            .stderr(contains(
                "val flag expects one of [\"RED\", \"BLUE\", \"GREEN\"]",
            ))
            .failure();
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
            .stdout(contains(format!(
                "{} {}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            )))
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
            .stderr(contains("error: unexpected argument '--version' found"))
            .failure();
    }
}

mod required_option_flag {
    use inkjet::parser::build_command_structure;

    use super::*;

    #[test]
    fn properly_runs_when_required_option_is_supplied() {
        let contents = r#"
## required_val
**OPTIONS**
* val
    * flag: --val
    * type: string
    * required
~~~bash
echo "Value: $val"
~~~
~~~powershell
param (
    $in = $env:val
)
Write-Output "Value: $in"
~~~
"#;

        let tree =
            build_command_structure(contents, true).expect("failed to build required option tree");
        let required_val_command = &tree
            .subcommands
            .iter()
            .find(|cmd| cmd.name == "required_val")
            .expect("serve command missing");
        assert!(!required_val_command.script.source.is_empty());
        let the_flag = required_val_command.named_flags.first().unwrap();
        assert!(the_flag.name == "val");
        assert!(the_flag.required);
    }

    #[test]
    fn errors_when_val_is_not_supplied() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## required_val
**OPTIONS**
* val
    * flag: --val
    * type: string
    * required
~~~bash
echo "This shouldn't render"
~~~
~~~powershell
Write-Output "This shouldn't render"
~~~
"#,
        );

        common::run_inkjet(&inkfile_path)
            .cli("required_val")
            .assert()
            .stderr(contains(
                "error: the following required arguments were not provided:\n  --val <val>",
            ))
            .failure();
    }
}

mod optional_args {
    use predicates::boolean::PredicateBooleanExt;

    use super::*;

    #[test]
    fn runs_with_optional_args() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## with_opt (required) (optional?)

~~~bash
echo "$required" "$optional"
~~~

~~~powershell
param(
    $req = $env:required,
    $opt = $env:optional
)

Write-Output "$req $opt"
~~~
"#,
        );

        common::run_inkjet(&inkfile_path)
            .cli("with_opt")
            .arg("I am required")
            .arg("I am optional")
            .assert()
            .stdout(contains("I am required I am optional"))
            .success();
    }

    #[test]
    fn does_not_fail_when_optional_arg_is_not_present() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## with_opt (required) (optional?)

~~~bash
echo "$required" "$optional"
~~~

~~~powershell
param(
    $req = $env:required,
    $opt = $env:optional
)

Write-Output "$req $opt"
~~~
"#,
        );

        common::run_inkjet(&inkfile_path)
            .cli("with_opt")
            .arg("I am required")
            .assert()
            .stdout(contains("I am optional").not())
            .success();
    }
}
