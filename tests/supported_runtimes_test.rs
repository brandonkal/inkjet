use assert_cmd::prelude::*;
use predicates::str::contains;

mod common;
use common::InkjetCommandExt;

#[test]
fn errors_when_no_lang_code_is_specified() {
    let (_temp, inkfile_path) = common::inkfile(
        r#"
## missing
~~~
echo "this won't do anything..."
~~~
"#,
    );

    common::run_inkjet(&inkfile_path)
        .command("missing")
        .assert()
        .code(1)
        .stderr(contains(
            "Command script requires a language code or shebang which determines which executor to use.",
        ))
        .failure();
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
