#!/home/brandon/mask/target/release/mask

> Development tasks for mask

## echo (name) (optional=default)
> Echo something

**OPTIONS**
* file
    * flags: -f --file
    * type: number
    * desc: Echo description

```sh
echo "Hello $name! Optional arg is $optional. File is $file"
echo "Task complete."
```

## short
> Echo something

**OPTIONS**
* flags: -f --file |string| Only run tests from a specific filename

```sh
echo "Hello $name! Optional arg is $optional. File is $file"
echo "Task complete."
```

## run (maskfile_command)

> Build and run mask in development mode

**NOTE:** This uses `cargo run` to build and run `mask` in development mode. You must have a `maskfile` in the current directory (this file) and must supply a valid command for that `maskfile` (`maskfile_command`) in order to test the changes you've made to `mask`. Since you can only test against this `maskfile` for now, you can add subcommands to the bottom and run against those instead of running one of the existing commands.

**EXAMPLE:** `mask run "test -h"` - outputs the help info of this `test` command

**OPTIONS**
* watch
    * flags: -w --watch
    * desc: Rebuild on file change

~~~bash
if [[ $watch == "true" ]]; then
    watchexec --exts rs --restart "cargo run -- $maskfile_command"
else
    cargo run -- $maskfile_command
fi
~~~



## build

> Build a release version of mask

~~~bash
cargo build --release
~~~



## link

> Build mask and replace your globally installed version with it for testing

~~~bash
cargo install --force --path .
~~~



## test

> Run all tests

**OPTIONS**
* file
    * flags: -f --file
    * type: string
    * desc: Only run tests from a specific filename

~~~bash
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
~~~



## deps

> Commands related to cargo dependencies

### deps upgrade

> Update the cargo dependencies

~~~bash
cargo update
~~~



## format

> Format all source files

**OPTIONS**
* check
    * flags: -c --check
    * desc: Show which files are not formatted correctly

~~~bash
if [[ $check == "true" ]]; then
    cargo fmt --all -- --check
else
    cargo fmt
fi
~~~



## lint//\_default

> Lint the project with clippy

~~~bash
cargo clippy
~~~
