% inkjet(1) Version 2.0.0 | Create interactive CLIs and execute Markdown with inkjet

<p align="center">
  <img height="180" width="180" src="https://user-images.githubusercontent.com/4714862/80295323-cf9e8180-8726-11ea-9919-2bbe1de7f5e5.png">
</p>

# NAME

inkjet - Create interactive CLIs and execute Markdown

# SYNOPSIS

| **inkjet** \[\-\-interactive]\[\-\-preview]\[\-\-help]\[\-\-version]\[\-\-inkfile=inkjet.md] _subcommand_

# DESCRIPTION

With `inkjet` you can build interactive CLIs from your existing Markdown. These CLIs can be as simple as a list of common tasks such as `test`, `build`, and `lint` or as complex applications with subcommands, flags, and options. All of this defined in a simple Markdown file that is both a **human-readable document** and a **command definition**! Your code is documentation and your documentation is code. Because Markdown is documentation focused, the format encourages descriptive text. This allows others to easily get started with your project's development setup by simply reading your `inkjet.md`.

# USE CASES

## Rapid CLI development

Inkjet is the fastest path to building and distributing custom command line apps. Because you ship a single Markdown file, you can distribute it with any web server. You can quickly convert your bash aliases to a CLI interface or use multiple languages to build an advanced app. It is all wrapped up with your documentation in a single readable file.

## Project specific tasks

You have a project with a bunch of random build and development scripts or an unwieldy `Makefile`. Simplify by having a single, readable file for your team members to add and modify existing tasks.

## Global system utility

You want a global utility CLI for various system tasks such as backing up directories or renaming multiple files. This is easily possible by making a bash alias for `inkjet --inkfile ~/my-global-inkjet.md`.

## Executable interactive tutorials and guides

Blog posts and tutorials often contain text with blocks of code walking the reader through the process of creating or installing an application or using a project. Markdown is commonly used as the authoring format for these guides. Using Inkjet's interactive capabilities, it is simple to take an existing tutorial and distribute an executable documentation file. Users can download your tutorial and read it, preview code blocks, and execute each step inside their terminals.

## Motivation

As developers, we create great new things. We use iterative development to reach new targets. But often the README.md falls behind. In other areas, we work to automate as much as possible. Using tools such as Ansible, Docker, Kubernetes, and Terraform, we aim for "infrastructure as code." Doing this means our infrastructure is well documented, and we can create and teardown environments quickly.

But with all these tools, project set up becomes more complex. Gone are the days of just a `git clone`. The "Getting Started" section of our READMEs resemble long lists of manual tasks that a new developer is expected to execute, typically by copying and pasting snippets of code into the terminal.

My last young project currently has a README.md that started out as a pasteboard of code snippets to run to reproduce an environment. The document expands with the project. It is reproducible, but it is not yet automated. Using it requires a lot of reading and copy-and-paste into the terminal. Eventually, I had planned to move these scripts into a combination of Ansible playbooks, Makefiles, and shell scripts. Automation! But those new scripts would then require additional Markdown documentation on how to use them.

Open-source projects that I respect have extremely complex README files. We automate everything else. Why not the bootstrap process?

That's when it occurred to me -- we need executable documentation. Markdown is well suited for this task because it renders well nearly everywhere with syntax highlighting of code blocks.

Inkjet helps you automate those complex (and currently manual) bootstrap guides.

We can replace Makefiles with inkjet.md files. Inkjet supports many languages, so you can write your tasks in Go, Ruby, TypeScript, bash, Python, etc. More extravagant interpreters are supported by using a shebang at the start of your code block.

Here's the [inkjet.md](/inkjet.md) that `inkjet` uses to build itself and run tests!

To get started, follow the guide below or check out the more [advanced features](#features "Features") `inkjet` has such as **positional args**, **named flags**, **subcommands**, other **scripting runtimes** and more!

# INSTALLATION

### Using Homebrew

Homebrew is the preferred method to install `inkjet` and keep it updated on macOS.

```
brew install brandonkal/tap/inkjet
```

### Debian Repository

[My Debian repository](https://git.kalinow.ski/kiterun/-/packages/debian/inkjet) is the preferred way to install `inkjet` on Linux. It includes shell completions and the man page. See how to configure the debian package here and then run `sudo apt install inkjet`

### Pre-compiled binaries for Linux, macOS, and Windows

Head to the Releases and look for the latest published version. Under **Assets** you'll see zips available for download for Linux, macOS, and Windows. Once downloaded, you can extract them and then move the `inkjet` binary to somewhere accessible in your `$PATH` like `mv inkjet /usr/local/bin`.

### From source

If you prefer to build from source, clone this repo. The entire build script is in [inkjet.md](/inkjet.md). The Linux build can also be run via the Earthfile script.

# GETTING STARTED

First, define a simple `inkjet.md` in your project root.

````markdown
# Tasks For My Project

> (Optional) Information entered here will appear in your CLI's about help text.

<!-- A heading defines the command's name -->

## build

<!-- An optional blockquote defines the command's description -->

> Builds my project

<!-- A code block defines the script to be executed -->

```sh
echo "building project..."
```

## test

> Tests my project

You can also write documentation anywhere you want. Only certain types
of Markdown patterns are parsed to determine the command structure.

```js
console.log("running project's tests");
```
````

Note this code block above is defined as js. By default, Inkjet supports sh, bash, zsh, fish, dash, JavaScript (node),
Python, Ruby, PHP, Go ([yaegi](https://github.com/traefik/yaegi)), and TypeScript ([deno](https://deno.com)) as scripting runtimes! Using shebang syntax, you can use any other interpreter to execute your scripts.

Then, try running one of your commands!

```sh
inkjet build
inkjet test
```

# FEATURES

## Interactive execution mode

Prefixing a subcommand with the interactive flag `-i` executes the command interactively.

In interactive mode:

1. The command's Markdown is rendered to the terminal as rich text support.
2. If any flags or options are specified in the spec, Inkjet will prompt the user for those parameters.
3. The user will be given the option to execute the step or preview the code block.
4. Required parameters will have "\*". If a default exits it will be shown in the prompt. Hitting enter will select the default.

Interactive execution mode is useful for tutorial guides or when you are not sure what options or flag parameters are required.

## Preview mode

Prefix a subcommand with the preview flag `-p` to extract code from the specified task's code block. If [bat](https://github.com/sharkdp/bat) is available, it will be used to pretty print the block with syntax highlighting using your installed theme. This mode is also useful for copying the block into the pasteboard: `inkjet -p build | pbcopy`.

## Shell completions

Inkjet can generate completions for your shell dynamically. In this way, you'll get helpful tab completions depending on the Markdown file. See the `completions` folder for the bash and fish scripts. For example the fish script calls `inkjet inkjet-dynamic-completions fish | source`. Your shell may cache results in the short term but generally you can navigate to different folders and get project-specific shell-completion.

## Positional arguments

These are defined beside the command name within `(round_brackets)`. They are required arguments that must be supplied for the command to run. An argument may be made optional by including a question mark: `(optional_arg?)`. The argument name is injected into the script's scope as an environment variable. Defaults can be set with an equals sign: `(port=8080)`. An arg with a default is naturally optional as well.

**Example:**

````markdown
## test (file) (test_case?)

> Run tests

```bash
echo "Testing $test_case in $file"
```

## serve (port=8080)

```
python -m SimpleHTTPServer $port
```
````

## Infinite arguments

An argument can be made to accept infinite arguments by including three dots: `(extra...)`. It can be made optional by including a question mark. This is best left for the last argument. Infinite args are collected as a space-separated string, perfect for shell expansion.

**Example:**

````markdown
## test (extra_args...?)

```
cargo test $extra_args
```
````

## Named flags

You can define a list of named flags for your commands. The flag name is injected into the script's scope as an environment variable. If a flag name includes a `-` it will be replaced with an underscore (i.e. `--no-color` becomes `no_color`)

It is important to note that `inkjet` always injects a very common `boolean` flag called `verbose` into every single command even if it's not declared. This saves a bit of typing for you! This means every command implicitly has a `-v` and `--verbose` flag available. Like all `boolean` flags, the value of the `$verbose` environment variable is either `"true"` or simply unset/non-existent.

**Example:**

````markdown
## serve

> Serve this directory

<!-- You must define OPTIONS right before your list of flags -->
<!-- Bold is used here for readability but is not required -->

**OPTIONS**

- port
  - flag: -p --port
  - type: string
  - desc: Which port to serve on
  - required

```sh
PORT=${port:-8080} # Set a fallback port if not supplied

if [[ "$verbose" == "true" ]]; then
    echo "Starting an http server on PORT: $PORT"
fi
python -m SimpleHTTPServer $PORT
```

## short (name)

> A shorthand syntax is also supported

The above flag can be specified in a single line.

OPTIONS

- flag: -p --port |string| required Which port to serve on

```sh
echo "Hello $name! Port is $port"
echo "Task complete."
```
````

You can also make your flag expect a numerical value by setting its `type` to `number`. `inkjet` will automatically validate it as a number for you. If it fails to validate, `inkjet` will exit with a helpful error message.

Flags are optional by default. Note that adding the word "required" to the flag list or shorthand definition will mark the flag a required parameter.

**Example:**

````markdown
## purchase (price)

> Calculate the total price of something.

**OPTIONS**

- tax
  - flag: -t --tax
  - type: number
  - desc: What's the tax?

```sh
TAX=${tax:-1} # Fallback to 1 if not supplied
echo "Total: $(($price * $TAX))"
```
````

## Subcommands

Nested command structures can easily be created since they are simply defined by the level of Markdown heading. H2 (`##`) is where you define your top-level commands. Every level after that is a subcommand. The only requirement is that subcommands must have all ancestor commands present in their heading.

**Example:**

````markdown
## services

> Commands related to starting, stopping, and restarting services

### services start (service_name)

> Start a service.

```bash
echo "Starting service $service_name"
```

### services stop (service_name)

> Stop a service.

```bash
echo "Stopping service $service_name"
```

#### services stop all

> Stop everything.

```bash
echo "Stopping everything"
```
````

## Hidden Subcommands

Simply prefix a subcommand's name with an underscore to make that command hidden. It will not be included in the generated CLI help pages.

This is useful for blocks of code that need to be shared by several tasks but should not define a visible user-callable command.

## Aliases and default target

Separate a subcommand name with `//` to define an alias.

````markdown
## lint//default

> Lint the project with clippy

```sh
cargo clippy
```
````

In the above example, simply calling `inkjet` with no arguments will call the lint command.

## Support for other scripting runtimes

On top of shell/bash scripts, `inkjet` also supports using node,
Python, Ruby, PHP, yaegi, and deno as scripting runtimes. This gives you the freedom to choose the right tool for the specific task at hand. For example, let's say you have a `serve` command and a `snapshot` command. You could choose python to `serve` a simple directory and maybe node to run a puppeteer script that generates a png `snapshot` of each page. If required, you can even specify a custom shebang.

For Python, Linux and Mac will search for a `python3` binary while on Windows, Inkjet expects a `python` binary.

**Example:**

````markdown
## shell (name)

> An example shell script

Valid lang codes: sh, bash, zsh, fish... any shell that supports -c

```zsh
echo "Hello, $name!"
```

## node (name)

> An example node script

Valid lang codes: js, javascript

```js
const { name } = process.env;
console.log(`Hello, ${name}!`);
```

## python (name)

> An example python script

Valid lang codes: py, python

```python
import os
name = os.getenv("name", "WORLD")
print("Hello, " + name + "!")
```

## ruby (name)

> An example ruby script

Valid lang codes: rb, ruby

```ruby
name = ENV["name"] || "WORLD"
puts "Hello, #{name}!"
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

## php (name)

> An example php script

```php
$name = getenv("name") ?: "WORLD";
echo "Hello, " . $name . "!\n";
```

## yaml

> An example yaml script using a custom shebang

While YAML is typically not executable, you could use shebangs to invoke kubectl, docker-compose, or Ansible.

```yaml
#!/usr/bin/env ansible-playbook
- name: This is a hello-world example
  tasks:
    - name: Hello
      copy:
        content: hello world
        dest: /tmp/testfile.txt
```
````

## Windows support

If bash is available in your PATH (for example via Git Bash), Inkjet can use it. Alternatively, you can add Powershell, Batch, or Cmd code blocks alongside the Linux/macOS code block. Depending on which platform this runs on, the correct code block will be executed.

**Example:**

````markdown
## link

> Build and link the binary globally

```bash
cargo install --force --path .
```

```powershell
[Diagnostics.Process]::Start("cargo", "install --force --path .").WaitForExit()
```
````

## Automatic help and usage output

You don't have to spend time writing out help info manually. `inkjet` uses your command descriptions and options to automatically generate help output. For every command, it adds the `-h` and `--help` flags.

**Example:**

```sh
inkjet services start -h
inkjet services start --help
```

All output the same help info:

```txt

Start a service.

Usage: inkjet services start [OPTIONS] <service_name>

Arguments:
  <service_name>  

Options:
  -v, --verbose  Sets the level of verbosity
  -h, --help     Print help
  -r, --restart  Restart this service if it's already running
  -w, --watch    Restart a service on file change
```

## Directives

You can change how parsing occurs by including some special directives in the Markdown file. Most of the time, I don't use these but they are available to you for advanced use cases.

### inkjet_sort: false

By default, subcommands in the help output are listed in the same order
they are defined in the Markdown file. Users can decide to instead have subcommands sorted alphabetically by defining this directive. As an example, if you are using inkjet to distribute a CLI of code snippets, sorted help would make sense. For projects, you may want the order to be as defined (e.g., publish comes after test).

### inkjet_fixed_dir: false

When you run an inkjet command from a project subdirectory, inkjet will by default search up the tree to find an `inkjet.md` file. In order for commands to work as expected, scripts execute as if their working directory was the same as the location of the `inkjet.md` file that defined them. Similarly, if you call Inkjet with `--inkfile tests/inkjet.md`, your commands will execute as if the working directory was `tests`. If this is not desired, simply include the `inkjet_fixed_dir: false` directive in the file to have the working directory match your current directory.

### inkjet_import: all

It's often the case that large projects will have multiple `inkjet.md` files.
For instance, each service may have its own `inkjet.md` file to define how to build and test that component. To enable the import feature, include the text directive `inkjet_import: all` somewhere within your main `inkjet.md` file. If Inkjet discovers this directive in the text, it will find all other `inkjet.md` files within the current folder and merge them together before parsing and building out the command tree. If the imported file has a H1 heading, its commands will appear as a subcommand of that heading. If only H2 and below headings are available in the imported file, those commands will become sibling commands for the parent. See [a merged example here](tests/merged-example.md).

The merge behavior is as follows:

1. Locate `inkjet.md` files and files ending in `.inkjet.md` within the current folder.
2. Found `inkjet.md` files are first sorted by directory depth and then alphabetically.
3. Merged definitions can override previously defined definitions.

The override behavior is useful as it enables you to share generic commands, and then override the generic on a project-by-project basis.

All imported `inkjet.md` files are run as if they were called directly. Namely, if `inkjet_fixed_dir` is not set to false, imported commands will run with their working directory set to the parent directory of its `inkjet.md` file.

**Example:**

````bash
$ tree
.
├── frontend
│   ├── Dockerfile
│   └── inkjet.md
└── inkjet.md
$ cat inkjet.md
inkjet_import: all
# main service
## release
```
echo "Release"
```
$ cat frontend/inkjet.md
# frontend
## build
```
echo "Building frontend"
docker build . -t frontend
```
$ inkjet frontend build
Building frontend...
...successful docker build output here...
$ inkjet release
Release
```
````

Note that in the above example `.` (period) works because the docker build is executed from frontend directory.

## Running Inkjet from within a script

You can easily call `inkjet` within scripts if you need to chain commands together. However, if you plan on [running inkjet with a different inkfile](#), you should consider using the `$INK` utility (documented below) instead which allows your scripts to be location-agnostic.

Shell scripts execute as if `set -e` is set.

**Example:**

````markdown
## bootstrap

> Installs deps, builds, links, migrates the db and then starts the app

```sh
inkjet install
inkjet build
inkjet link
# $INK also works. It's an alias variable for `inkjet --inkfile <path_to_inkfile>`
# which guarantees your scripts will still work even if they are called from
# another directory.
$INK db migrate
$INK start
```
````

## Exit Codes

If your command exits with an error, `inkjet` will exit with its status code. This allows you to chain commands which will exit on the first error.

**Example:**

````markdown
## ci

> Runs tests and checks for lint and formatting errors

```sh
inkjet test \
    && inkjet lint \
    && inkjet format --check
```
````

Normally, inkjet transparently returns the exit code of your command. However if inkjet itself experiences an error, you may see one of these errors:

| Status Code | Cause                                                                      |
|-------------|----------------------------------------------------------------------------|
|      2      | Invalid command line args                                                  |
|      5      | I/O error (i.e. unable to merge inkjet.md files, executer cannot be found) |
|      66     | inkjet.md file not found or empty                                          |
|      78     | inkjet config error (i.e. markdown is invalid)                             |

## Running inkjet with a different inkfile

If you're in a directory that doesn't have a `inkjet.md` but you want to reference one somewhere else, you can with the `--inkfile <path_to_inkfile>` option.

**Example:**

```sh
inkjet --inkfile ~/inkjet.md <subcommand>
```

**Tip:** Make a bash alias for this so you can call it anywhere easily

```bash
# Call it something fun
alias snippet="inkjet --inkfile ~/inkjet.md"

# You can run this from anywhere
snippet <subcommand>
```

**Tip:** The shorthand alternative to `--inkfile` is `-c`. This flag also accepts the text contents of the inkfile or `-` to read stdin. This allows for using Inkjet as an interpreter similar to other shells. If the value of this flag contains multiple lines, it is interpreted as the contents. Otherwise, it is parsed as a filename as usual.

```bash
inkjet -c "$(cat inkjet.md)"
```

# ENVIRONMENT

Inside each script's execution environment, `inkjet` injects a few environment variable helpers. Scripts inherit the environment from your shell.

**`$INK`**

This is useful when [running inkjet within a script](#running-inkjet-from-within-a-script "Running Inkjet from within a script"). This variable allows us to call `$INK command` instead of `inkjet --inkfile <path> command` inside scripts so that they can be location-agnostic (not care where they are called from). This is especially handy for global inkfiles which you may call from anywhere.

**`$INKJET`**

This is similar to `INK` above, except it always resolves to the original `inkjet.md` file. For instance, You may have some common scripts in the project's main `inkjet.md` file and call those scripts in imported `inkjet.md` files throughout the project. Note that if you call the imported inkfile directly, it will resolve the same as `$INK` above. For this reason, you will see different behavior depending on where you call the script.

**`$INK_DIR`**

This variable is an absolute path to the inkfile's parent directory. Having the parent directory available allows us to load files relative to the inkfile itself, which can be useful when you have commands that depend on other external files.

**`$INKJET_DIR`**

This is much like `INK_DIR` except it always resolves to the main `inkjet.md` file's parent directory.

**`$INKET_IMPORTED`**

A helper utility that is set to "true" if the script was imported by another `inkjet.md` file.

**`$NO_COLOR`**

Inkjet respects NO_COLOR to disable colorized output for its own commands.

# LICENSE

Inkjet is Copyright © 2020 Brandon Kalinowski
Inkjet is based on the work in mask (MIT) Copyright © 2019 Jake Deichert

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE
OR OTHER DEALINGS IN THE SOFTWARE.

# AUTHOR

Copyright © 2020 [Brandon Kalinowski](https://brandonkalinowski.com).

Source code available on [GitHub](https://github.com/brandonkal/inkjet)
