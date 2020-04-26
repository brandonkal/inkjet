<p align="center">
  <img height="180" width="180" src="https://user-images.githubusercontent.com/4714862/80295323-cf9e8180-8726-11ea-9919-2bbe1de7f5e5.png">
</p>

`inkjet` is a CLI task runner which is defined by a simple markdown file. It searches for a `inkjet.md` in the current directory which it then parses for commands and arguments.

A `inkjet.md` is both a **human-readable document** and a **command definition**! Being documentation focused allows others to easily get started with your project's development setup by simply reading your `inkjet.md`. A nice advantage of using markdown is that syntax highlighting for code blocks is built-in to many editors and renderers like GitHub itself.

Here's the [inkjet.md](/inkjet.md) that `inkjet` itself uses as an example!

To get started, follow the guide below or check out the more [advanced features](#features) `inkjet` has like **positional args**, **optional flags**, **subcommands**, other **scripting runtimes** and more!


## Installation

### Precompiled binaries for linux and macOS

Head to the [Releases page][releases] and look for the latest published version. Under **Assets** you'll see zips available for download for linux and macOS. Once downloaded, you can unzip them and then move the `inkjet` binary to somewhere accessible in your `$PATH` like `mv inkjet /usr/local/bin`.

### Cargo

`inkjet` is published to [crates.io][crate] which allows you to install it via `cargo install inkjet`.

### From source

If you prefer to build from source, clone this repo and then run `cargo build --release`





## Getting started

First, define a simple `inkjet.md` in your project.

```markdown
# Tasks For My Project


<!-- A heading defines the command's name -->
## build

<!-- A blockquote defines the command's description -->
> Builds my project

<!-- A code block defines the script to be executed -->
~~~sh
echo "building project..."
~~~


## test

> Tests my project

You can also write documentation anywhere you want. Only certain types of markdown patterns
are parsed to determine the command structure.

Note this code block below is defined as js. So far, inkjet supports node,
python, ruby and php as scripting runtimes!

~~~js
console.log("running project's tests")
~~~
```

Then, try running one of your commands!

~~~sh
inkjet build
inkjet test
~~~





## Features

### Positional arguments

These are defined beside the command name within `(round_brackets)`. They are required arguments that must be supplied for the command to run. [Optional args][2] are coming soon. The argument name is injected into the script's scope as an environment variable.

**Example:**

```markdown
## test (file) (test_case)

> Run tests

~~~bash
echo "Testing $test_case in $file"
~~~
```

### Optional flags

You can define a list of optional flags for your commands. The flag name is injected into the script's scope as an environment variable.

Important to note that `inkjet` auto injects a very common `boolean` flag called `verbose` into every single command even if it's not used. This saves a bit of typing for you! This means every command implicitly has a `-v` and `--verbose` flag already. The value of the `$verbose` environment variable is either `"true"` or simply unset/non-existent.

**Example:**

```markdown
## serve

> Serve this directory

<!-- You must define OPTIONS right before your list of flags -->
**OPTIONS**
* port
    * flags: -p --port
    * type: string
    * desc: Which port to serve on

~~~sh
PORT=${port:-8080} # Set a fallback port if not supplied

if [[ "$verbose" == "true" ]]; then
    echo "Starting an http server on PORT: $PORT"
fi
python -m SimpleHTTPServer $PORT
~~~
```

You can also make your flag expect a numerical value by setting its `type` to `number`. This means `inkjet` will automatically validate it as a number for you. If it fails to validate, `inkjet` will exit with a helpful error message.

**Example:**

```markdown
## purchase (price)

> Calculate the total price of something.

**OPTIONS**
* tax
    * flags: -t --tax
    * type: number
    * desc: What's the tax?

~~~sh
TAX=${tax:-1} # Fallback to 1 if not supplied
echo "Total: $(($price * $TAX))"
~~~
```

### Subcommands

Nested command structures can easily be created since they are simply defined by the level of markdown heading. H2 (`##`) is where you define your top-level commands. Every level after that is a subcommand. The only requirement is that subcommands must have all ancestor commands present in their heading.

**Example:**
```markdown
## services

> Commands related to starting, stopping, and restarting services

### services start (service_name)

> Start a service.

~~~bash
echo "Starting service $service_name"
~~~

### services stop (service_name)

> Stop a service.

~~~bash
echo "Stopping service $service_name"
~~~

#### services stop all

> Stop everything.

~~~bash
echo "Stopping everything"
~~~
```

### Support for other scripting runtimes

On top of shell/bash scripts, `inkjet` also supports using node, python, ruby and php as scripting runtimes. This gives you the freedom to choose the right tool for the specific task at hand. For example, let's say you have a `serve` command and a `snapshot` command. You could choose python to `serve` a simple directory and maybe node to run a puppeteer script that generates a png `snapshot` of each page.

**Example:**

```markdown
## shell (name)

> An example shell script

Valid lang codes: sh, bash, zsh, fish... any shell that supports -c

~~~zsh
echo "Hello, $name!"
~~~


## node (name)

> An example node script

Valid lang codes: js, javascript

~~~js
const { name } = process.env;
console.log(`Hello, ${name}!`);
~~~


## python (name)

> An example python script

Valid lang codes: py, python

~~~python
import os
name = os.getenv("name", "WORLD")
print("Hello, " + name + "!")
~~~


## ruby (name)

> An example ruby script

Valid lang codes: rb, ruby

~~~ruby
name = ENV["name"] || "WORLD"
puts "Hello, #{name}!"
~~~


## php (name)

> An example php script

~~~php
$name = getenv("name") ?: "WORLD";
echo "Hello, " . $name . "!\n";
~~~
```

### Automatic help and usage output

You don't have to spend time writing out help info manually. `inkjet` uses your command descriptions and options to automatically generate help output. For every command, it adds `-h, --help` flags and an alternative `help <name>` command.

**Example:**
~~~sh
inkjet services start -h
inkjet services start --help
inkjet services help start
inkjet help services start
~~~

All output the same help info:

~~~txt
inkjet-services-start
Start or restart a service.

USAGE:
    inkjet services start [FLAGS] <service_name>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Sets the level of verbosity
    -r, --restart    Restart this service if it's already running
    -w, --watch      Restart a service on file change

ARGS:
    <service_name>
~~~

### Running inkjet from within a script

You can easily call `inkjet` within scripts if you need to chain commands together. However, if you plan on [running inkjet with a different inkfile](#running-inkjet-with-a-different-inkfile), you should consider using the `$INKJET` utility instead which allows your scripts to be location-agnostic.

**Example:**

```markdown
## bootstrap

> Installs deps, builds, links, migrates the db and then starts the app

~~~sh
inkjet install
inkjet build
inkjet link
# $INKJET also works. It's an alias variable for `inkjet --inkfile <path_to_inkfile>`
# which guarantees your scripts will still work even if they are called from
# another directory.
$INKJET db migrate
$INKJET start
~~~
```

### Inherits the script's exit code

If your command exits with an error, `inkjet` will exit with its status code. This allows you to chain commands which will exit on the first error.

**Example:**

```markdown
## ci

> Runs tests and checks for lint and formatting errors

~~~sh
inkjet test \
    && inkjet lint \
    && inkjet format --check
~~~
```

### Running inkjet with a different inkfile

If you're in a directory that doesn't have a `inkjet.md` but you want to reference one somewhere else, you can with the `--inkfile <path_to_inkfile>` option.

**Example:**

~~~sh
inkjet --inkfile ~/inkjet.md <subcommand>
~~~

**Tip:** Make a bash alias for this so you can call it anywhere easily

~~~bash
# Call it something fun
alias wask="inkjet --inkfile ~/inkjet.md"

# You can run this from anywhere
wask <subcommand>
~~~

### Environment variable utilities

Inside of each script's execution environment, `inkjet` injects a few environment variable helpers that might come in handy.

**`$INKJET`**

This is useful when [running inkjet within a script](#running-inkjet-from-within-a-script). This variable allows us to call `$INKJET command` instead of `inkjet --inkfile <path> command` inside scripts so that they can be location-agnostic (not care where they are called from). This is especially handy for global inkfiles which you may call from anywhere.

**`$INKJETFILE_DIR`**

This variable is an absolute path to the inkfile's parent directory. Having the parent directory available allows us to load files relative to the inkfile itself which can be useful when you have commands that depend on other external files.





## Upcoming features

* [ ] [Optional (non-required) positional arguments][2]
* [ ] [Infinite positional args](https://github.com/jakedeichert/inkjet/issues/4)





## Use cases

Here's some example scenarios where `inkjet` might be handy.

### Project specific tasks

You have a project with a bunch of random build and development scripts or an unwieldy `Makefile`. You want to simplify by having a single, readable file for your team members to add and modify existing tasks.


### Global system utility

You want a global utility CLI for a variety of system tasks such as backing up directories or renaming a bunch of files. This is easily possible by making a bash alias for `inkjet --inkfile ~/my-global-inkjet.md`.





## FAQ

### Windows support?

Currently, this is [unknown][windows_issue]. I'm pretty sure the executor logic will need to be adjusted for Windows. Git Bash and Ubuntu on Windows have been reported to work but they are not actively being tested.

### Is `inkjet` available as a lib?

`inkjet` was designed as a lib from the beginning and is accessible. However, it's very undocumented and will need to be cleaned up before it's considered stable.

### Where did the inspiration come from?

I'm definitely not the first to come up with this idea of using markdown as a CLI structure definition.

My frustrations with `make`'s syntax is what led me to search for other options. I landed on [just][just] for awhile which was a pretty nice improvement. My favourite feature of `just` is its support for other language runtimes, which is why `inkjet` also has this ability! However, it still didn't have some features I wanted like nested subcommands and multiple optional flags.

At some point in my searching, I came across [maid][maid] which is where most of the inspiration for `inkjet` comes from. I thought it was brilliant that markdown could be used as a command definition format while still being so readable.

So why did I choose to rebuild the wheel instead of using `maid`? For one, I preferred installing a single binary, like `just` is, rather than installing an npm package with hundreds of deps. I also had a few ideas on how I could improve upon `maid` which is why `inkjet` supports multiple levels of nested subcommands as well as optional flags and positional args. Also... I just really wanted to build another thing with Rust :)

I also need to mention [clap][clap] and [pulldown-cmark][cmark] which are really the core parts of `inkjet` that made it so easy to create.





## Contributing

Check out our [Contribution Guidelines](CONTRIBUTING.md) before creating an issue or submitting a PR 🙌

Also, please review and follow the rules within our [Code of Conduct](CODE_OF_CONDUCT.md) 🙂





## Author

Jake Deichert with the help of contributors.

[@jakedeichert][twitter] on Twitter · [Website][website]




[github_ci]: https://github.com/jakedeichert/inkjet/actions?query=workflow%3ACI
[crate]: https://crates.io/crates/inkjet
[releases]: https://github.com/jakedeichert/inkjet/releases
[new_issue]: https://github.com/jakedeichert/inkjet/issues/new
[website]: https://jakedeichert.com
[twitter]: https://twitter.com/jakedeichert
[2]: https://github.com/jakedeichert/inkjet/issues/5
[maid]: https://github.com/egoist/maid
[just]: https://github.com/casey/just
[clap]: https://github.com/clap-rs/clap
[cmark]: https://github.com/raphlinus/pulldown-cmark
[windows_issue]: https://github.com/jakedeichert/inkjet/issues/10
