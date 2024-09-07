#!/usr/bin/env inkjet

> Development tasks for inkjet

inkjet_import: all

## echo (name) (optional=default)

> Echo something

**OPTIONS**

- file
  - flag: -n --num
  - type: number
  - desc: Echo description

```
echo "Hello $name! Optional arg is $optional. File is $file"
echo "Task complete."
```

## short

> Echo something

**OPTIONS**

- flag: -f --file |string| Only run tests from a specific filename
- flag: -nc --no-check Test with dash replacement

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
  - flag: -w --watch
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

### build mac

> Build a release version of inkjet for mac arm and x86

```bash
set -x
zips_dir=$INKJET_DIR/output/zips
mkdir -p $zips_dir
rm -r $zips_dir/* || true
version=$($INKJET utils v crate)
_build() {
  cargo build --target $arch --release
  target_dir=$INKJET_DIR/target/$arch/release
  cp $INKJET_DIR/README.md $target_dir
  cp -r $INKJET_DIR/completions $target_dir
  cp $INKJET_DIR/output/inkjet.1 $target_dir # regenerate with `earthly --artifact +man/inkjet.1 ./output/inkjet.1`
  cd $target_dir
  strip inkjet
  tar -czf $zips_dir/inkjet-v$version-$arch.tar.gz inkjet inkjet.1 completions README.md
}

arch=aarch64-apple-darwin
_build
arch=x86_64-apple-darwin
_build
cd $zips_dir
shasum -a 256 * > $zips_dir/checksums.sha256.txt
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
	fmt.Println("Hello from Go")
}
```

## yaml

> An example yaml script using custom shebang

While YAML is typically not executable, you can use shebangs to invoke kubectl, docker-compose, or Ansible.

```yaml
#!/usr/bin/env ansible-playbook
- name: This is a hello-world example
  hosts: localhost
  tasks:
    - name: Hello
      copy:
        content: hello world
        dest: /tmp/testfile.txt
```

## test

> Run all tests

**OPTIONS**

- file
  - flag: -f --file
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
  - flag: -c --check
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
import "./tests/imported.tsx";
const five: number = 5;
console.log(five);
console.log(JSON.stringify(Deno.args));
```

## opts (name) (optional?)

> Echo something

**OPTIONS**

- file
  - flag: -f --file
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

- flag: -s --string |string| First option
- flag: --bool Second option
- flag: --number |number| Enter a number

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

> Run tests in coverage profile and generate a coverage report

```sh
rm -rf target/coverage || :
$INKJET cov build
$INKJET cov build # view.rs shows incorrect coverage if run once
$INKJET cov collect
```

### cov build

> Run tests to build coverage report. This may need to be run several times

```sh
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
export RUSTDOCFLAGS="-Cpanic=abort"
cargo +nightly build --profile coverage
export LLVM_PROFILE_FILE="your_name-%p-%m.profraw"
cargo +nightly test --profile coverage
```

### cov collect

> Collect coverage report as HTML

```sh
v="${EARTHLY_GIT_SHORT_HASH:-$(inkjet utils v)}"
echo "Version is: $v"
rm -rf target/cov || :
rm target/cov.zip || :
mkdir -p target/cov || :
zip -0 target/cov.zip `find target/coverage \( -name "inkjet*.gc*" \) -print`
grcov target/cov.zip -s . -t lcov --llvm --ignore-not-existing --ignore "/*" -o target/cov/intermediate.info \
  --excl-start '#\[cfg\(test\)\]' --excl-stop '#\[cfg\(cov:end-exclude\)\]'
rm target/cov.zip
rust-covfix target/cov/intermediate.info -o target/lcov.info
sed -i "1s/.*/TN:inkjet_$v/" target/lcov.info
sed -i "1s/-/_/" target/lcov.info
genhtml -o target/cov/ --show-details --highlight --ignore-errors source  --title "inkjet-$v" \
  --legend target/lcov.info --no-function-coverage
mv target/cov/src/* target/cov
echo "Coverage report generated at target/cov" >&2
```

## utils

### utils v

> Returns a version string of the worker-image based on if it is dirty

```sh
revision=$(git log -1 --format=%h)
if git status --porcelain >/dev/null 2>&1; then
  revision="$revision-dirty"
fi
echo "$revision"
```

#### utils v crate

> Returns the version per the crate

```
cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "inkjet") | .version'
```

## double-dash -- (extra)

> A test to ensure the double dash functions

```sh
echo $_
echo Done
```

## man

> Preview README as man page for distribution

```
pandoc README.md -s -t man --lua-filter=man-filter.lua | man -l -
```
