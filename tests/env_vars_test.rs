// Copyright 2020 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

use assert_cmd::prelude::*;
use predicates::str::contains;

mod common;
pub use common::InkjetCommandExt;
pub use common::*;

// NOTE: This test suite depends on the inkjet binary being available in the current shell

// Using current_dir(common::temp_path()) to make sure the default inkjet.md can't be found
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

~~~powershell
$path = $env:INKJET.replace("\\?\", "")
$pos = $path.IndexOf(" ");
$arglist = $path.Substring($pos + 1);

Start-Process inkjet.exe -ArgumentList "$arglist test" -wait -NoNewWindow -PassThru
~~~

## test

~~~bash
echo "tests passed"
~~~

~~~powershell
Write-Output "tests passed"
~~~
"#,
        );

        common::run_inkjet(&inkfile_path)
            .current_dir(common::temp_path())
            .command("ci")
            .assert()
            .stdout(contains("tests passed"))
            .success();
    }

    #[test]
    fn set_to_the_correct_value1() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## run

~~~bash
echo "inkjet = $INKJET"
~~~
"#,
        );

        let pattern1 = "inkjet = ";

        #[cfg(not(windows))]
        let pattern2 = "inkjet --inkfile /";

        #[cfg(windows)]
        let pattern2 = "inkjet.exe --inkfile \\\\?\\C:\\Users\\User\\AppData\\Local\\Temp\\";

        common::run_inkjet(&inkfile_path)
            .current_dir(common::temp_path())
            .command("run")
            .assert()
            // Absolute inkfile path starts with /
            .stdout(contains(pattern1))
            .stdout(contains(pattern2))
            // And ends with inkjet.md
            .stdout(contains("inkjet.md"))
            .success();
    }
}

// Using current_dir(common::temp_path()) to make sure the default inkjet.md can't be found
mod env_var_inkfile_dir {
    use super::*;

    #[test]
    fn set_to_the_correct_value2() {
        let (_temp, inkfile_path) = common::inkfile(
            r#"
## run

~~~bash
echo "inkfile_dir = $INKJET_DIR"
~~~
"#,
        );

        #[cfg(not(windows))]
        let pattern = "inkfile_dir = /";

        #[cfg(windows)]
        let pattern = "inkfile_dir = \\\\?\\C:\\Users\\User\\AppData\\Local\\Temp";

        common::run_inkjet(&inkfile_path)
            .current_dir(common::temp_path())
            .command("run")
            .assert()
            // Absolute inkfile path starts with /
            .stdout(contains(pattern))
            .success();
    }
}
