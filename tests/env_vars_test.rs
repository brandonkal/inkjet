use assert_cmd::prelude::*;
use predicates::str::contains;

mod common;
use common::InkjetCommandExt;

// NOTE: This test suite depends on the inkjet binary being available in the current shell

// Using current_dir(".github") to make sure the default inkjet.md can't be found
mod env_var_inkjet {
    use super::*;

    #[test]
    fn works_from_any_dir() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## ci

~~~bash
$INKJET test
~~~

## test

~~~bash
echo "tests passed"
~~~
"#,
        );

        common::run_inkjet(&inkfile_path)
            .current_dir(".github")
            .command("ci")
            .assert()
            .stdout(contains("tests passed"))
            .success();
    }

    #[test]
    fn set_to_the_correct_value() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## run

~~~bash
echo "inkjet = $INKJET"
~~~
"#,
        );

        common::run_inkjet(&inkfile_path)
            .current_dir(".github")
            .command("run")
            .assert()
            // Absolute inkfile path starts with /
            .stdout(contains("inkjet = inkjet --inkfile /"))
            // And ends with inkjet.md
            .stdout(contains("inkjet.md"))
            .success();
    }
}

// Using current_dir(".github") to make sure the default inkjet.md can't be found
mod env_var_inkfile_dir {
    use super::*;

    #[test]
    fn set_to_the_correct_value() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## run

~~~bash
echo "inkfile_dir = $INKJET_DIR"
~~~
"#,
        );

        common::run_inkjet(&inkfile_path)
            .current_dir(".github")
            .command("run")
            .assert()
            // Absolute inkfile path starts with /
            .stdout(contains("inkfile_dir = /"))
            .success();
    }
}
