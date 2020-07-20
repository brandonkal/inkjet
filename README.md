<p align="center">
  <img height="180" width="180" src="https://user-images.githubusercontent.com/4714862/80295323-cf9e8180-8726-11ea-9919-2bbe1de7f5e5.png">
</p>

How long is your README.md?

As developers, we are great at creating new things and using iterative development to reach new targets. But sometimes the README.md falls behind. In other areas, we work to automate as much as possible. Using tools such as Ansible, Docker, Kubernetes, and Terraform, we aim for "infrastructure as code." Doing this means our infrastructure is well documented and we can create and teardown environments quickly.

But with all these tools, project set up becomes more complex. Gone are the days of just a `git clone`. The "Getting Started" section of our READMEs resemble long lists of manual tasks that a new developer is expected to execute, often by copying and pasting snippets of code into the terminal.

My last young project currently has a README.md that started out as a pasteboard of code snippets to run to reproduce an environment. The document expands with the project. It is reproducible, but it is not yet automated. Eventually I had planned to move these scripts into a combination of Ansible playbooks, Makefiles, and shell scripts. That would likely then require additional markdown documentation on how to use all of those scripts.

An open source project that I respect has an extremely complex README file. Is it not odd that while we automate everything in the infrastructure space, we still fill our bootstrap guides with complex manual processes?

That's when it occurred to me -- we need executable documentation. Markdown is well suited for this task because it renders well nearly everywhere with syntax highlighting of code blocks.

We can replace Makefiles with inkjet.md files. Inkjet supports many languages, so you can write your tasks in Go, Ruby, TypeScript, bash, etc. More extravagant interpreters are supported by using a shebang at the start of your code block.

With `inkjet` you can build interactive CLIs from your existing markdown. These CLIs can be as simple as a list of common tasks such as `test`, `build`, and `lint` or as complex applications with subcommands, flags, and options. All of this defined in a simple markdown file that is both a **human-readable document** and a **command definition**! Your code is documentation and your documentation is code. Because markdown is documentation focused, the format encourages descriptive text. This allows others to easily get started with your project's development setup by simply reading your `inkjet.md`.

In its current state, inkjet works really well as a command runner for projects and sharing snippets as CLIs. In the fullness of time, I hope to expand inkjet to work as a general-purpose executable markdown tool.

Here's the [inkjet.md](/inkjet.md) that `inkjet` uses to build itself and run tests!

To get started, follow the guide below or check out the more [advanced features](#features) `inkjet` has such as **positional args**, **optional flags**, **subcommands**, other **scripting runtimes** and more!

## Installation

### Using Homebrew

Homebrew is the preferred method to install `inkjet` and keep it updated on macOS and Linux.

```
brew install brandonkal/tap/inkjet
```

### Pre-compiled binaries for linux and macOS

Head to the [Releases page][releases] and look for the latest published version. Under **Assets** you'll see zips available for download for linux and macOS. Once downloaded, you can unzip them and then move the `inkjet` binary to somewhere accessible in your `$PATH` like `mv inkjet /usr/local/bin`.

### From source

If you prefer to build from source, clone this repo and then run `cargo build --release`. The entire build script is in `inkjet.md`.

## Getting started

First, define a simple `inkjet.md` in your project.

````markdown
# Tasks For My Project

<!-- A heading defines the command's name -->

> (Optional) Information entered here will appear in the CLI about help text.

## build

<!-- A blockquote defines the command's description -->

> Builds my project

<!-- A code block defines the script to be executed -->

```sh
echo "building project..."
```

## test

> Tests my project

You can also write documentation anywhere you want. Only certain types
of markdown patterns are parsed to determine the command structure.

```js
console.log("running project's tests")
```
````

Note this code block above is defined as js. By default, inkjet supports sh, bash, zsh, fish, dash, JavaScript (node),
Python, Ruby, PHP, Go (yaegi), and TypeScript (deno) as scripting runtimes! Using shebang syntax, you can use any other interpreter to execute your scripts.

Then, try running one of your commands!

```sh
inkjet build
inkjet test
```

## Name

Why the name inkjet?

- I needed a name that is short and could work with a short alias. I have `alias i=inkjet` in my bashrc. This works well: `i test`, `i build`.
- Inkjet printers made desktop publishing economical and fast. In the same way, **brandonkal/inkjet** makes building a CLI for project tasks fast and economical.
- Like the printer, it is well suited for documentation.
- The name was available. I needed a filename that identifies itself to what it does. Looking through GitHub, there are only a couple of repositories that contain an inkjet.md file and those have to do with the printer.

## Features

### Interactive execution mode

Prefixing a subcommand with the interactive flag `-i` allows users to execute a command interactively.

In interactive mode:

1. The task's markdown is rendered in the terminal as rich text with image and link support.
2. If any flags or options are specified in the spec, inkjet will prompt the user for those parameters.
3. The user will be given the option to execute the step or preview the code block.

Interactive execution mode is useful for tutorial guides or when you are not sure what options or flags parameters are required.

### Preview mode

Prefix a subcommand with the preview flag `-p` to extract code from the specified task's code block. If bat is available, it will be used to pretty print the block with syntax highlighting. This mode is also useful for copying the block into the pasteboard: `inkjet -p build | pbcopy`.

### Positional arguments

These are defined beside the command name within `(round_brackets)`. They are required arguments that must be supplied for the command to run. An argument may be made optional by including a question mark: `(optional_arg?)`. The argument name is injected into the script's scope as an environment variable.

**Example:**

````markdown
## test (file) (test_case?)

> Run tests

```bash
echo "Testing $test_case in $file"
```
````

### Optional flags

You can define a list of optional flags for your commands. The flag name is injected into the script's scope as an environment variable. If a flag name includes a `-` it will be replaced with an underscore (i.e. `--no-color` becomes `no_color`)

It is important to note that `inkjet` auto injects a very common `boolean` flag called `verbose` into every single command even if it's not used. This saves a bit of typing for you! This means every command implicitly has a `-v` and `--verbose` flag already. The value of the `$verbose` environment variable is either `"true"` or simply unset/non-existent.

**Example:**

````markdown
## serve

> Serve this directory

<!-- You must define OPTIONS right before your list of flags -->
<!-- Bold is used here for readability but is not required -->

**OPTIONS**

- port
  - flags: -p --port
  - type: string
  - desc: Which port to serve on

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

- flags: -p --port |string| Which port to serve on

```sh
echo "Hello $name! Port is $port"
echo "Task complete."
```
````

You can also make your flag expect a numerical value by setting its `type` to `number`. This means `inkjet` will automatically validate it as a number for you. If it fails to validate, `inkjet` will exit with a helpful error message.

**Example:**

````markdown
## purchase (price)

> Calculate the total price of something.

**OPTIONS**

- tax
  - flags: -t --tax
  - type: number
  - desc: What's the tax?

```sh
TAX=${tax:-1} # Fallback to 1 if not supplied
echo "Total: $(($price * $TAX))"
```
````

### Subcommands

Nested command structures can easily be created since they are simply defined by the level of markdown heading. H2 (`##`) is where you define your top-level commands. Every level after that is a subcommand. The only requirement is that subcommands must have all ancestor commands present in their heading.

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

### Hidden Subcommands

Simply prefix a subcommand's name with an underscore to make that command hidden. It will not be included in the generated CLI help pages.

This is useful for blocks of code that need to shared for several tasks but should not define its own user-callable command.

### Aliases and default target

Separate a subcommand name with `//` to define an alias.

````markdown
## lint//default

> Lint the project with clippy

```sh
cargo clippy
```
````

In the above example, simply calling `inkjet` with no arguments will call the lint command.

### Support for other scripting runtimes

On top of shell/bash scripts, `inkjet` also supports using node, python, ruby, go, deno, and php as scripting runtimes. This gives you the freedom to choose the right tool for the specific task at hand. For example, let's say you have a `serve` command and a `snapshot` command. You could choose python to `serve` a simple directory and maybe node to run a puppeteer script that generates a png `snapshot` of each page.

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
const { name } = process.env
console.log(`Hello, ${name}!`)
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

> An example yaml script using custom shebang

While YAML is typically not executable, you can use shebangs to invoke kubectl, docker-compose, or Ansible.

```yaml
#!/usr/bin/env ansible-playbook -i ../hosts -K
- name: This is a hello-world example
  tasks:
    - name: Hello
      copy:
        content: hello world
        dest: /tmp/testfile.txt
```
````

### Automatic help and usage output

You don't have to spend time writing out help info manually. `inkjet` uses your command descriptions and options to automatically generate help output. For every command, it adds `-h, --help` flags and an alternative `help <name>` command.

**Example:**

```sh
inkjet services start -h
inkjet services start --help
inkjet services help start
inkjet help services start
```

All output the same help info:

```txt
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
```

### Directives

You can change how parsing occurs by including some special directives in the markdown file.

#### inkjet_sort: false

By default subcommands in the help output are listed in the same order
they are defined in the markdown file. Users can choose to instead have subcommands sorted alphabetically by defining this directive. As an example, if you are using inkjet to distribute a CLI of code snippets, sorted help would make sense. For projects, you may want the order to be as defined (e.g. publish comes after test).

#### inkjet_fixed_dir: false

When you run an inkjet command from a project subdirectory, inkjet will by default search up the tree to find a `inkjet.md` file. In order for commands to work as expected, scripts execute as if their working directory was the same as the location of the `inkjet.md` file that defined them. Similarly, if you call inkjet with `--inkfile tests/inkjet.md`, your commands will execute as if the working directory was `tests`. If this is not desired, simply include the `inkjet_fixed_dir: false` directive in the file to have the working directory match your current directory.

#### inkjet_import: all

It's often the case that large projects will have multiple `inkjet.md` files.
For instance, each service may have its own `inkjet.md` file to define how to build and test that component. To enable the import feature, include the text directive `inkjet_import: all` somewhere within your main `inkjet.md` file. If inkjet discovers this directive in the text, it will run a shell command that finds all other `inkjet.md` files within the current folder and merge them together before parsing and building out the command tree. If the imported file has an h1 heading, its commands will appear as a subcommand of that heading. If only h2 and below headings are available in the imported file, those commands will become sibling commands for the parent. See [a merged example here](tests/merged-example.md).

The merge behavior is as follows:

1. Found `inkjet.md` files are first sorted by directory depth and then alphabetically.
2. Merged definitions can override previously-defined definitions.

The override behavior is useful as it enables you to share generic commands, and them override them on a project-by-project basis as needed.

All imported inkjet.md files are run as if they were called directly. Namely, if `inkjet_fixed_dir` is not set to false, imported commands will run with their working directory set to the parent directory of its `inkjet.md` file.

**Example:**

````bash
$ tree
.
├── frontend
│   ├── Dockerfile
│   └── inkjet.md
└── inkjet.md
$ cat inkjet.md
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

Note that the above example `.` works because as the docker build is run from frontend directory.

````

### Running inkjet from within a script

You can easily call `inkjet` within scripts if you need to chain commands together. However, if you plan on [running inkjet with a different inkfile](#running-inkjet-with-a-different-inkfile), you should consider using the `$INK` utility instead which allows your scripts to be location-agnostic.

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

### Inherits the script's exit code

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

### Running inkjet with a different inkfile

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

**Tip:** The shorthand alternative to `--inkfile` is `-c`. This flag also accepts the text contents of the inkfile or `-` to read stdin. This is enabled in order to use inkjet as an interpreter similar to other shells. If the value of this flag contains a multiple lines, it is interpreted as the contents, otherwise it is parsed as a filename as usual.

```bash
inkjet -c "$(cat inkjet.md)"
```

### Environment variable utilities

Inside of each script's execution environment, `inkjet` injects a few environment variable helpers that might come in handy.

**`$INK`**

This is useful when [running inkjet within a script](#running-inkjet-from-within-a-script). This variable allows us to call `$INK command` instead of `inkjet --inkfile <path> command` inside scripts so that they can be location-agnostic (not care where they are called from). This is especially handy for global inkfiles which you may call from anywhere.

**`$INKJET`**

This is similar to `INK` above but it always resolves to the orinal `inkjet.md` file. For instance, You may have some common scripts in the project's main `inkjet.md` file and call those scripts in imported `inkjet.md` files throughout the project. Note that if you call the imported inkfile directly, it will resolve the same as `$INK` above. For this reason, you will see different behavior depending on where you call the script.

**`$INK_DIR`**

This variable is an absolute path to the inkfile's parent directory. Having the parent directory available allows us to load files relative to the inkfile itself which can be useful when you have commands that depend on other external files.

**`$INKJET_DIR`**

This is much like `INK_DIR` but it always resolves to the main `inkjet.md` file's parent directory.

## Use cases

Here's some example scenarios where `inkjet` might be handy.

### Project specific tasks

You have a project with a bunch of random build and development scripts or an unwieldy `Makefile`. Simplify by having a single, readable file for your team members to add and modify existing tasks.

### Global system utility

You want a global utility CLI for a variety of system tasks such as backing up directories or renaming a bunch of files. This is easily possible by making a bash alias for `inkjet --inkfile ~/my-global-inkjet.md`.

### Rapid CLI development

While inkjet is suitable for project tasks, it can also be used to build and distribute custom command line apps. Because you ship a markdown file, you can distribute it with any web server.

### Executable interactive tutorials and guides

Blog posts and tutorials often contain text with blocks of code walking the reader through the process of creating or installing an application or using a project. Markdown is commonly used as the authoring format for these guides. Using inkjet's interactive capabilities, it is simple to take an existing tutorial and distribute an executable documentation file. Users can download your tutorial and read it, preview code blocks, and execute each step inside their terminals.

## FAQ

### Windows support?

Currently, this is unknown. I'm pretty sure the executor logic will need to be adjusted for Windows. Git Bash and Ubuntu on Windows have been reported to work but they are not actively being tested.

### Is `inkjet` available as a lib?

`inkjet` was designed as a lib from the beginning and is accessible. However, it's very undocumented and will need to be cleaned up before it's considered stable.

## Contributing

Check out our [Contribution Guidelines](CONTRIBUTING.md) before creating an issue or submitting a PR 🙌

### Wishlist

If inkjet is useful to you, please consider authoring one of these features.

1. Import support. Using markdown links, we can import and combine several inkjet files together. For instance, you can link to your main `inkjet.md` file and define project-specific tasks and overrides in the project-specific `inkjet.md` file.
2. Investigate dependency management. The one thing we lose migrating from Makefiles is dependency tracking. Most of my makefiles are filled with .PHONY, but having tasks specify their dependencies is still a welcome option.
3. Compile markdown file to bash or POSIX script a la mdsh.
4. Ability to execute any markdown file that contains code blocks, stepping through each section.

## Author

Brandon Kalinowski. This is based on the mask project by [Jake Deichert](https://github.com/jakedeichert).
This started as a fork of that project and I've added many features such as aliases, interactive execution, preview mode, optional arguments, dash support, default shell with `set -e`, a fixed working directory by default, golang support, shebang support, and more.

This is my first foray into the realm of Rust programming.

[Website](https://brandonkalinowski.com)
