#!/usr/bin/env inkjet

> Development tasks for inkjet

inkjet_import: all

## echo (name) (optional=default)

> Echo something

**OPTIONS**

- file
  - flags: -n --num
  - type: number
  - desc: Echo description

```
echo "Hello $name! Optional arg is $optional. File is $file"
echo "Task complete."
```

## short

> Echo something

**OPTIONS**

- flags: -f --file |string| Only run tests from a specific filename
- flags: -nc --no-check Test with dash replacement

```sh
echo "Hello $name! Optional arg is $optional. File is $file"
echo "${no_check:-false}"
echo "Task complete."
```

## run (inkfile_command)

> Build and run inkjet in development mode

**NOTE:** This uses `cargo run` to build and run `inkjet` in development mode. You must have a `inkfile` in the current directory (this file) and must supply a valid command for that `inkfile` (`inkfile_command`) in order to test the changes you've made to `inkjet`. Since you can only test against this `inkfile` for now, you can add subcommands to the bottom and run against those instead of running one of the existing commands.

**EXAMPLE:** `inkjet run "test -h"` - outputs the help info of this `test` command

**OPTIONS**

- watch
  - flags: -w --watch
  - desc: Rebuild on file change

```bash
if [[ $watch == "true" ]]; then
    watchexec --exts rs --restart "cargo run -- $inkfile_command"
else
    cargo run -- $inkfile_command
fi
```

## build

> Build a release version of inkjet

```bash
cargo build --release
cd $INKJET_DIR/target/release
strip inkjet
platform=`echo $(uname -s) | tr '[:upper:]' '[:lower:]'`
tar -czf inkjet-${platform}.tar.gz inkjet
```

## ping

> Echo pong for ping tests

```sh
echo "blip"
```

## ping

> Echo pong override for ping tests

Later definitions override previous definitions.

```sh
echo "pong"
```

## link

> Build inkjet and replace your globally installed version with it for testing

```bash
cargo install --force --path .
```

## go

> Execute embedded Go scripts with yaegi

```go
package main

import "fmt"

func main() {
	fmt.Println("hello from go")
}
```

## test

> Run all tests

**OPTIONS**

- file
  - flags: -f --file
  - type: string
  - desc: Only run tests from a specific filename

```bash
extra_args=""

if [[ "$verbose" == "true" ]]; then
    # Run tests linearly and make logs visible in output
    extra_args="-- --nocapture --test-threads=1"
fi

echo "Running tests..."
if [[ -z "$file" ]]; then
    # Run all tests by default
    cargo test $extra_args
else
    # Tests a specific integration filename
    cargo test --test $file $extra_args
fi
echo "Tests passed!"
```

## deps

> Commands related to cargo dependencies

### deps upgrade

> Update the cargo dependencies

```bash
cargo update
```

## format

> Format all source files

**OPTIONS**

- check
  - flags: -c --check
  - desc: Show which files are not formatted correctly

```bash
if [[ $check == "true" ]]; then
    cargo fmt --all -- --check
else
    cargo fmt
fi
```

## shebang

> Run a shebang as a script

```
#!/bin/sh
echo "Hello World!"
```

## lint//default//check

> Lint the project with clippy

```bash
cargo clippy
```

## deno

```ts
import './tests/imported.tsx'
const five: number = 5
console.log(five)
console.log(JSON.stringify(Deno.args))
```

## opts (name) (optional?)

> Echo something

**OPTIONS**

- file
  - flags: -f --file
  - type: string
  - desc: Only run tests from a specific filename

```sh
echo "Hello $name! Optional arg is $optional. File is $file"
echo "Task complete."
```

## interactive (one) (two) (three?)

> Test interactive mode with three positional args and three flags

Run this with and without specific options specified.

**OPTIONS**

- flags: -s --string |string| First option
- flags: --bool Second option
- flags: --number |number| Enter a number

```sh
echo "The values are one=$one two=$two three=$three"
echo "The flag values are string=$string bool=$bool number=$number"
```

## fixed-directory

If this directive is set to false, scripts will always execute as if called from the inkjet file's location.

inkjet_fixed_dir: true

> Test to ensure fixed directory works

```
ls .vscode
```

## cov

> Collect coverage report as HTML

```bash
pkg="inkjet"
cargo test --no-run || exit $?
rm -rf target/cov target/cov-tmp

for file in target/debug/*; do
  if [[ -f "$file" && -x "$file" ]]; then
    folder="target/cov-tmp/$(basename "$file")"
    mkdir -p "$folder"
    kcov --exclude-pattern=/.cargo,/usr/lib,/tests --exclude-region='#[cfg(test)]:#[cfg(testkcovstopmarker)]' "$folder" "$file"
  fi
done
kcov --exclude-pattern=/.cargo,/usr/lib,/tests --exclude-region='#[cfg(test)]:#[cfg(testkcovstopmarker)]' --exclude-line='@kcov-ignore' --merge target/cov-tmp/merged target/cov-tmp/*
mv target/cov-tmp/merged/kcov-merged target/cov
rm -rf target/cov-tmp
echo "Coverage report generated at target/cov" >&2
```

## gcov-build

```sh
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
export RUSTDOCFLAGS="-Cpanic=abort"
cargo build
cargo test
```

## grcov

```sh
grcov ./target/debug -s . -t lcov --llvm --branch \
  --ignore /.cargo,/usr/lib,tests
  --ignore-not-existing -o ./target/debug/lcov.info
genhtml -o target/debug/report --show-details --highlight \
 --ignore-errors source --legend ./target/debug/lcov.info
```

## install

### install kcov-deps

```
sudo apt-get install libcurl4-openssl-dev libelf-dev libdw-dev cmake gcc
```

### install kcov

```
wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz
tar xzf master.tar.gz
cd kcov-master
mkdir build
cd build
cmake ..
make
sudo make install
```
