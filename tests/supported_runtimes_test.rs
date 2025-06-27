// Copyright 2020 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

use assert_cmd::prelude::*;
use predicates::str::contains;

mod common;
pub use common::*;

#[test]
fn executes_when_no_lang_code_is_specified() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## missing
~~~
echo "this will execute..."
~~~
"#,
    );

    common::run_inkjet(&inkfile_path)
        .command("missing")
        .assert()
        .stdout(contains("this will execute..."))
        .success();
}

#[cfg(windows)]
#[test]
fn powershell() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## powershell
~~~powershell
param (
    $name = $env:name
)

Write-Output "Hello, $name!"
~~~
"#,
    );

    common::run_inkjet(&inkfile_path)
        .command("powershell")
        .env("name", "World")
        .assert()
        .stdout(contains("Hello, World!"))
        .success();
}

#[cfg(windows)]
#[test]
fn batch() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## batch
~~~batch
echo "Hello, %name%!"
~~~
"#,
    );

    common::run_inkjet(&inkfile_path)
        .command("batch")
        .env("name", "World")
        .assert()
        .stdout(contains("Hello, World!"))
        .success();
}

#[cfg(windows)]
#[test]
fn cmd() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## cmd
~~~cmd
echo "Hello, %name%!"
~~~
"#,
    );

    common::run_inkjet(&inkfile_path)
        .command("cmd")
        .env("name", "World")
        .assert()
        .stdout(contains("Hello, World!"))
        .success();
}

#[test]
fn sh() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## sh
~~~sh
echo Hello, $name!
~~~
"#,
    );

    common::run_inkjet(&inkfile_path)
        .command("sh")
        .env("name", "World")
        .assert()
        .stdout(contains("Hello, World!"))
        .success();
}

#[test]
fn bash() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## bash
~~~bash
echo Hello, $name!
~~~
"#,
    );

    common::run_inkjet(&inkfile_path)
        .command("bash")
        .env("name", "World")
        .assert()
        .stdout(contains("Hello, World!"))
        .success();
}

#[test]
fn node() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## node
~~~js
const { name } = process.env;
console.log(`Hello, ${name}!`);
~~~
"#,
    );

    common::run_inkjet(&inkfile_path)
        .command("node")
        .env("name", "World")
        .assert()
        .stdout(contains("Hello, World!"))
        .success();
}

#[test]
fn python() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## python
~~~py
import os
name = os.getenv("name", "WORLD")
print("Hello, " + name + "!")
~~~
"#,
    );

    common::run_inkjet(&inkfile_path)
        .command("python")
        .env("name", "World")
        .assert()
        .stdout(contains("Hello, World!"))
        .success();
}

#[test]
fn ruby() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## ruby
~~~ruby
name = ENV["name"] || "WORLD"
puts "Hello, #{name}!"
~~~
"#,
    );

    common::run_inkjet(&inkfile_path)
        .command("ruby")
        .env("name", "World")
        .assert()
        .stdout(contains("Hello, World!"))
        .success();
}

#[test]
fn php() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## php
~~~php
$name = getenv("name") ?: "WORLD";

echo "Hello, " . $name . "!\n";

~~~
"#,
    );

    common::run_inkjet(&inkfile_path)
        .command("php")
        .env("name", "World")
        .assert()
        .stdout(contains("Hello, World!"))
        .success();
}

#[test]
fn missing_runtime() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## missing
```missingbinary
$name = getenv("name") ?: "WORLD";

echo "Hello, " . $name . "!\n";

```
"#,
    );

    common::run_inkjet(&inkfile_path)
        .command("missing")
        .assert()
        // NOTE: we are also checking that color output works here
        .stderr(contains(
            "\x1b[31m\x1b[4mERROR (inkjet):\x1b[39m\x1b[24m Please check if missingbinary is installed to run the command.\nNo such file or directory (os error 2)",
        ))
        .failure();
}

#[test]
fn typescript_deno() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## check (name)
```ts
const name: string = Deno.env.get("name")!;
console.log(`Hello ${name}!`);
```
"#,
    );
    common::run_inkjet(&inkfile_path)
        .command("check")
        .arg("Brandon")
        .assert()
        .stdout(contains("Hello Brandon!"))
        .success();
}

#[test]
#[ignore]
#[cfg(not(windows))]
fn go() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## go
> Execute embedded Go scripts with yaegi

```go
package main

import "fmt"

func main() {
    fmt.Println("Hello from Go!")
}
```
"#,
    );
    common::run_inkjet(&inkfile_path)
        .command("go")
        .assert()
        .stdout(contains("Hello from Go!"))
        .success();
}

#[test]
#[cfg(not(windows))]
fn shebang() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## check
> Execute a shebang

```yaml
#!/usr/bin/env cat
message: Hello from YAML
```
"#,
    );
    common::run_inkjet(&inkfile_path)
        .command("check")
        .assert()
        .stdout(contains("Hello from YAML"))
        .success();
}
