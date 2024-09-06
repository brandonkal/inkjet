// Copyright 2020 Brandon Kalinowski (brandonkal)
// SPDX-License-Identifier: MIT

use colored::*;
use pulldown_cmark::CodeBlockKind::Fenced;
use pulldown_cmark::{
    Event::{Code, End, Html, Start, Text},
    Options, Parser, Tag,
};
use regex::Regex;
use std::collections::HashSet;

use crate::command::{Arg, CommandBlock, NamedFlag};

/// Creates the message that is returned on an error
fn invalid_type_msg(t: &str) -> String {
    format!("Invalid flag type '{}' Expected string | number | bool.", t)
}

/// The main inkjet markdown parsing logic. Takes an inkfile content as a string and returns the parsed CommandBlock tree.
pub fn build_command_structure(inkfile_contents: &str) -> Result<CommandBlock, String> {
    let parser = create_markdown_parser(inkfile_contents);
    let mut commands = vec![];
    let mut current_command = CommandBlock::new(1);
    let mut current_named_flag = NamedFlag::new();
    let mut text = "".to_string();
    let mut list_level = 0;
    let mut first_was_pushed = false;
    let mut current_file = "".to_string();
    let mut in_block_quote = false;

    for (event, range) in parser.into_offset_iter() {
        match event {
            Start(tag) => {
                match tag {
                    Tag::Heading(heading_level) => {
                        // Add the last command before starting a new one.
                        // Don't add the first command during the first iteration.
                        if heading_level > 1 || first_was_pushed {
                            first_was_pushed = true;
                            commands.push(current_command.build());
                        }
                        current_command = CommandBlock::new(heading_level as u8);
                        current_command.inkjet_file = current_file.clone();
                        current_command.start = range.start;
                    }
                    #[cfg(not(windows))]
                    Tag::CodeBlock(Fenced(lang_code)) => {
                        let lc = lang_code.to_string();
                        if lc != "powershell" && lc != "batch" && lc != "cmd" {
                            current_command.end = range.start;
                            current_command.script.executor = lc;
                        }
                    }
                    #[cfg(windows)]
                    Tag::CodeBlock(Fenced(lang_code)) => {
                        current_command.end = range.start;
                        current_command.script.executor = lang_code.to_string();
                    }
                    Tag::List(_) => {
                        // We're in an options list if the current text above it is "OPTIONS"
                        if text == "OPTIONS" || list_level > 0 {
                            list_level += 1;
                        }
                    }
                    Tag::BlockQuote => {
                        in_block_quote = true;
                    }
                    _ => (),
                };

                // Reset all state
                text = "".to_string();
            }
            End(tag) => match tag {
                Tag::Heading(heading_level) => {
                    let mut virtual_heading_level = heading_level;
                    if first_was_pushed && heading_level == 1 {
                        virtual_heading_level = 2; // This case occurs during a merge
                    }
                    let (name, aliases, args) =
                        parse_heading_to_cmd(virtual_heading_level, text.clone());
                    if name.is_empty() {
                        return Err("unexpected empty heading name".to_string());
                    }
                    if name.contains(char::is_whitespace) {
                        return Err(format!("Command names cannot contain spaces. Found '{}'. Did you forget to wrap args in ()?", name));
                    }
                    current_command.name = name;
                    current_command.args = args;
                    if !aliases.is_empty() {
                        current_command.aliases = aliases;
                    }
                }
                Tag::BlockQuote => {
                    if in_block_quote {
                        in_block_quote = false;
                    }
                }
                #[cfg(not(windows))]
                Tag::CodeBlock(Fenced(lang_code)) => {
                    let lc = lang_code.to_string();
                    if lc != "powershell" && lc != "batch" && lc != "cmd" {
                        current_command.script.source = text.to_string();
                    }
                }
                #[cfg(windows)]
                Tag::CodeBlock(_) => {
                    current_command.script.source = text.to_string();
                }
                Tag::List(_) => {
                    // Don't go lower than zero (for cases where it's a non-OPTIONS list)
                    list_level = std::cmp::max(list_level - 1, 0);
                    // Must be finished parsing the current option
                    if list_level == 1 {
                        // Add the current one to the list and start a new one
                        current_command.named_flags.push(current_named_flag.clone());
                        current_named_flag = NamedFlag::new();
                    }
                }
                _ => (),
            },
            Text(body) => {
                text += body.as_ref();

                if in_block_quote {
                    current_command.desc.push_str(body.as_ref());
                    current_command.desc.push(' ');
                }

                // Options level 1 is the flag name
                if list_level == 1 {
                    if text.contains(':') {
                        // Shorthand syntax
                        let mut flag_split = text.splitn(2, ':');
                        if flag_split.next().unwrap_or("").trim() == "flag" {
                            let val = flag_split.next().unwrap_or("").trim();
                            let mut desc_words = String::with_capacity(val.len());
                            for word in val.split_whitespace() {
                                if word.starts_with("--") {
                                    let name = word.split("--").collect::<Vec<&str>>().join("");
                                    current_named_flag.long = name.clone();
                                    current_named_flag.name = name;

                                // Must be a short flag name
                                } else if word.starts_with('-') {
                                    // Get the single char
                                    let name = word.get(1..2).unwrap_or("");
                                    current_named_flag.short = name.to_string();
                                } else if word.starts_with('|') && word.ends_with('|') {
                                    let mut kind = word.to_string();
                                    kind.pop();
                                    kind.remove(0);
                                    match kind.as_str() {
                                        "string" => {
                                            current_named_flag.takes_value = true;
                                        }
                                        "number" => {
                                            current_named_flag.takes_value = true;
                                            current_named_flag.validate_as_number = true;
                                        }
                                        "bool" | "boolean" => {}
                                        t => {
                                            return Err(invalid_type_msg(t));
                                        }
                                    }
                                } else if word == "required" {
                                    current_named_flag.required = true;
                                } else {
                                    desc_words.push(' ');
                                    desc_words.push_str(word)
                                }
                            }
                            current_named_flag.desc = desc_words.trim().to_string();
                        }

                        // Add the current one to the list and start a new one
                        current_command.named_flags.push(current_named_flag.clone());
                        current_named_flag = NamedFlag::new();
                    } else {
                        current_named_flag.name = text.clone();
                    }
                }
                // Options level 2 is the flag config
                else if list_level == 2 {
                    let mut config_split = text.splitn(2, ':');
                    let param = config_split.next().unwrap_or("").trim();
                    let val = config_split.next().unwrap_or("").trim();
                    match param {
                        "desc" => current_named_flag.desc = val.to_string(),
                        "type" => {
                            if val == "string" || val == "number" {
                                current_named_flag.takes_value = true;
                                if val == "number" {
                                    current_named_flag.validate_as_number = true;
                                }
                            } else {
                                return Err(invalid_type_msg(val));
                            }
                        }
                        // Parse out the short and long flag names
                        "flag" => {
                            let short_and_long_flags: Vec<&str> = val.splitn(2, ' ').collect();
                            for flag in short_and_long_flags {
                                // Must be a long flag name
                                if flag.starts_with("--") {
                                    let name = flag.split("--").collect::<Vec<&str>>().join("");
                                    current_named_flag.long = name;
                                }
                                // Must be a short flag name
                                else if flag.starts_with('-') {
                                    // Get the single char
                                    let name = flag.get(1..2).unwrap_or("");
                                    current_named_flag.short = name.to_string();
                                }
                            }
                        }
                        "choices" => {
                            current_named_flag.choices = val
                                .split(',')
                                .map(|choice| choice.trim().to_owned())
                                .collect();
                        }
                        "required" => {
                            current_named_flag.required = true;
                        }
                        _ => (),
                    };
                }
            }
            Html(html) => {
                // **Note:** Internally, inkjet uses a special comment in the form of
                // `<!-- inkfile: imported/inkjet.md -->` to set the working directory.
                // Users should consider not using this implementation detail directly
                // as it's not self documenting (comments are hidden when rendered).
                let s = "<!-- inkfile: ";
                if html.starts_with(s) {
                    current_file = html.replace(s, "").replace(" -->", "");
                }
                text += html.as_ref();
            }
            Code(inline_code) => {
                text += &format!("`{}`", inline_code);
                if in_block_quote {
                    current_command.desc.push_str(&format!("`{}`", inline_code));
                    current_command.desc.push(' ');
                }
            }
            _ => (),
        };
    }

    // Add the last command
    commands.push(current_command.build());

    // Convert the flat commands array and to a tree of subcommands based on level
    let all = treeify_commands(commands);
    let all = remove_duplicates(all);
    let root_command = all.first().expect("Inkjet: root command must exist");
    let has_duplicate_aliases = validate_no_duplicate_aliases(root_command.clone());
    if has_duplicate_aliases {
        return Err("Please update inkjet files to remove duplicate aliases".to_string());
    }
    // The command root
    Ok(root_command.clone())
}

fn validate_no_duplicate_aliases(cmd: CommandBlock) -> bool {
    let mut duplicates_found = false;
    let mut seen_aliases: HashSet<String> = HashSet::new();
    let mut errors: Vec<String> = Vec::new();

    if !cmd.subcommands.is_empty() {
        for subcommand in cmd.subcommands {
            let aliases = subcommand.aliases.split("//");
            for alias in aliases {
                if seen_aliases.contains(alias) {
                    duplicates_found = true;
                    errors.push(alias.to_string());
                    eprintln!(
                        "{} Duplicate command alias found: {}",
                        "ERROR (inkjet):".red(),
                        alias
                    );
                } else if !alias.is_empty() {
                    seen_aliases.insert(alias.to_string());
                }
            }
            if !subcommand.subcommands.is_empty() {
                duplicates_found = validate_no_duplicate_aliases(subcommand)
            }
        }
    }
    duplicates_found
}

// remove duplicate commands to enable override function
fn remove_duplicates(mut cmds: Vec<CommandBlock>) -> Vec<CommandBlock> {
    trait Dedup {
        fn clear_duplicates(&mut self);
    }

    // Implement Dedup for Vec<CommandBlock>
    impl Dedup for Vec<CommandBlock> {
        fn clear_duplicates(&mut self) {
            let mut already_seen = vec![];
            self.retain(|item| {
                if already_seen.contains(item) {
                    eprintln!(
                        "{} Duplicate command overwritten: {}",
                        "INFO (inkjet):".yellow(),
                        item.name
                    );
                    false
                } else {
                    already_seen.push(item.clone());
                    true
                }
            });
        }
    }
    cmds.reverse();
    cmds.clear_duplicates();
    cmds.reverse();
    for c in &mut cmds {
        if !c.subcommands.is_empty() {
            c.subcommands = remove_duplicates(c.subcommands.clone());
        }
    }
    cmds
}

fn create_markdown_parser(inkfile_contents: &str) -> Parser {
    // Set up options and parser. Strikethroughs are not part of the CommonMark standard
    // and we therefore must enable it explicitly.
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    Parser::new_ext(inkfile_contents, options)
}

fn trim_and_remove_options(input: &str) -> String {
    let trimmed = input.trim();
    let re = Regex::new(r"\s+").unwrap();
    let normalized = re.replace_all(trimmed, " ");
    if let Some(result) = normalized.strip_suffix("OPTIONS") {
        result.trim().to_string()
    } else {
        normalized.to_string()
    }
}

/// `treeify_commands` takes a flat vector of CommandBlocks and recursively builds a tree with subcommands as children.
/// It is called by the parser.
fn treeify_commands(commands: Vec<CommandBlock>) -> Vec<CommandBlock> {
    let mut command_tree = vec![];
    let mut current_command = commands.first().expect("command should exist").clone();
    let num_commands = commands.len();
    let mut add = 0;
    let mut allow_increment = false;

    #[allow(clippy::needless_range_loop, clippy::comparison_chain)]
    for i in 0..num_commands {
        let mut c = commands.get(i).unwrap().clone();
        let is_last_cmd = i == num_commands - 1;

        c.desc = trim_and_remove_options(&c.desc);

        // We allow virtually increasing the command level if multiple h1s are found with subcommands
        // This enables us to cat multiple inkjet.md files together and have it function as a larger CLI
        if c.cmd_level > 1 {
            allow_increment = true;
        }
        if allow_increment && c.cmd_level == 1 {
            add = 1;
        }
        c.cmd_level += add;

        // This must be a subcommand
        if c.cmd_level > current_command.cmd_level {
            current_command.subcommands.push(c.clone());
        }
        // This must be a sibling command
        // Make sure the initial command doesn't skip itself before it finds children
        else if c.cmd_level == current_command.cmd_level && i > 0 {
            // Found a sibling, so the current command has found all children.
            command_tree.push(current_command.clone());
            current_command = c.clone();
        }

        // The last command needs to be added regardless, since there's not another iteration of the loop to do so
        if is_last_cmd && c.cmd_level >= current_command.cmd_level {
            command_tree.push(current_command.clone());
        }
    }

    // Treeify all subcommands recursively
    for c in &mut command_tree {
        if !c.subcommands.is_empty() {
            c.subcommands = treeify_commands(c.subcommands.clone());
        }
    }

    // the command or any one of its subcommands must have script to be included in the tree
    // root level commands must be retained
    command_tree.retain(|c| c.script.has_script() || !c.subcommands.is_empty() || c.cmd_level == 1);

    command_tree
}

fn parse_heading_to_cmd(heading_level: u32, text: String) -> (String, String, Vec<Arg>) {
    // Anything after double dash is handled later -- if defined, it is the last arg
    let mut parts = text.split(" -- ");
    let main_text = parts.next().unwrap();
    // Why heading_level > 2? Because level 1 is the root command title (unused)
    // and level 2 can't be a subcommand so no need to split.
    let name = if heading_level > 2 {
        // Takes a subcommand name like this:
        // "#### db flush postgres (arg_name)"
        // and returns "postgres (arg_name)" as the actual name
        main_text
            .split_whitespace()
            .collect::<Vec<&str>>()
            // Get subcommand after the parent command name
            .split_at(heading_level as usize - 2)
            .1
            .join(" ")
    } else if heading_level == 1 {
        main_text.split_whitespace().next().unwrap().to_string()
    } else {
        main_text.to_string()
    };

    // Find any arguments. They look like this: (arg_name)
    let name_and_args: Vec<&str> = name.split(|c| c == '(' || c == ')').collect();
    let (name, args_split) = name_and_args.split_at(1);

    let name = name.join(" ");
    let mut name_and_alias = name.trim().splitn(2, "//");
    let name = match name_and_alias.next() {
        Some(n) => n.to_lowercase(),
        _ => "".to_string(), // cov:ignore
    };
    let alias = match name_and_alias.next() {
        Some(a) => a.to_lowercase(),
        _ => "".to_string(),
    };

    let mut out_args: Vec<Arg> = vec![];

    // Parse the arg strings and push results to output vector
    if !args_split.is_empty() {
        let args = args_split.join("");
        let args: Vec<&str> = args.split_whitespace().collect();
        for arg_str in args {
            out_args.push(parse_arg(arg_str));
        }
    }

    let caps = Regex::new(r"-- \(([^)]+)\)").unwrap().captures(&text);
    if caps.is_some() {
        let last_arg = caps
            .expect("Inkjet: regex should match for last arg")
            .get(1)
            .unwrap()
            .as_str();
        let mut parsed = parse_arg(last_arg);
        parsed.last = true;
        out_args.push(parsed);
    }

    (name, alias, out_args)
}

fn parse_arg(arg_str: &str) -> Arg {
    if arg_str.ends_with('?') {
        let mut arg = (*arg_str).to_lowercase();
        arg.pop(); // remove `?`
        if arg.ends_with('…') {
            arg.pop();
            Arg::new(arg, false, None, true)
        } else if arg.ends_with("...") {
            arg.pop();
            arg.pop();
            arg.pop();
            return Arg::new(arg, false, None, true);
        } else {
            return Arg::new(arg, false, None, false);
        }
    } else if arg_str.contains('=') {
        let parts: Vec<&str> = arg_str.splitn(2, '=').collect();
        // will always have >= 2 parts
        #[allow(clippy::indexing_slicing)]
        return Arg::new(
            parts[0].to_lowercase(),
            false,
            // All words are lowercased but the default
            Some(parts[1].to_string()),
            false,
        );
    } else if arg_str.ends_with('…') {
        let mut arg = (*arg_str).to_lowercase();
        arg.pop();
        return Arg::new(arg, true, None, true);
    } else if arg_str.ends_with("...") {
        let mut arg = (*arg_str).to_lowercase();
        arg.pop();
        arg.pop();
        arg.pop();
        return Arg::new(arg, true, None, true);
    } else {
        return Arg::new((*arg_str).to_lowercase(), true, None, false);
    }
}

#[cfg(test)]
const TEST_INKJETFILE: &str = r#"
# Document Title

This is an example inkfile for the tests below.

## serve (port) -- extra info

> Serve the app on the `port`

~~~bash
echo "Serving on port $port"
~~~


## node (name)

> An example node script

Valid lang codes: js, javascript

```js
const { name } = process.env;
console.log(`Hello, ${name}!`);
```


## no_script

This command has no source/script.

## no_space
> this should be the description
**OPTIONS**
* whatever
    * flag: -w --whatever
    * desc: Whatever

~~~bash
echo "the description is wrong..."
~~~

"#;

#[cfg(test)]
mod build_command_structure {
    use super::*;

    #[test]
    fn builds_boolean() {
        let tree = build_command_structure(
            r#"
## boolean

**OPTIONS**
- flag: -s --set |bool| Which port to serve on
~~~
echo $set
~~~
        "#,
        )
        .expect("build tree failed");
        let boolean_command = &tree
            .subcommands
            .iter()
            .find(|cmd| cmd.name == "boolean")
            .expect("boolean command missing");
        assert_eq!(boolean_command.name, "boolean");
        assert!(
            !boolean_command
                .named_flags
                .first()
                .expect("named flag not attached")
                .takes_value
        );
    }

    #[test]
    fn validates_string_and_removes_duplicate() {
        let tree = build_command_structure(
            r#"
## string

> Should be ignored
OPTIONS
- flag: -s --str |bool| A boolean
```
echo "Ignore me"
```

## string
OPTIONS
- flag: -s --str |string| A string
```
echo "the string is $str"
```
        "#,
        )
        .expect("build tree failed");
        let string_command = &tree
            .subcommands
            .iter()
            .find(|cmd| cmd.name == "string")
            .expect("string command missing");
        assert_eq!(string_command.name, "string");
        assert!(
            string_command
                .named_flags
                .first()
                .expect("named flag not attached")
                .takes_value
        );
        assert!(
            !string_command
                .named_flags
                .first()
                .expect("named flag not attached")
                .validate_as_number
        );
    }

    #[test]
    fn errors_on_duplicate_alias() {
        let result = build_command_structure(
            r#"
## first//default

> Should be ignored
OPTIONS
- flag: -s --str |bool| A boolean
```
echo "Ignore me"
```

## second//default
OPTIONS
- flag: -s --str |string| A string
```
echo "the string is $str"
```
        "#,
        );
        assert!(result.is_err());
        if let Err(ref message) = result {
            assert_eq!(
                message,
                "Please update inkjet files to remove duplicate aliases"
            );
        }
    }

    #[test]
    fn parses_serve_command_description() {
        let tree = build_command_structure(TEST_INKJETFILE).expect("build tree failed");
        let serve_command = &tree
            .subcommands
            .iter()
            .find(|cmd| cmd.name == "serve")
            .expect("serve command missing");
        assert_eq!(serve_command.desc, "Serve the app on the `port`");
    }

    #[test]
    fn parses_no_space_command_description() {
        let tree = build_command_structure(TEST_INKJETFILE).expect("build tree failed");
        let serve_command = &tree
            .subcommands
            .iter()
            .find(|cmd| cmd.name == "no_space")
            .expect("no_space command missing");
        assert_eq!(serve_command.desc, "this should be the description");
    }

    #[test]
    fn fails_if_name_has_spaces() {
        let file = r#"
## sub

### a b c

> description

```
echo "abc"
```
       "#;
        let tree_result = build_command_structure(file);
        if let Err(e) = tree_result {
            assert!(e == "Command names cannot contain spaces. Found 'b c'. Did you forget to wrap args in ()?",
                "Unexpected error message: \"{}\"", e);
        } else {
            panic!("expected a parse error");
        }
    }

    #[test]
    fn parses_serve_required_positional_arguments() {
        let tree = build_command_structure(TEST_INKJETFILE).expect("build tree failed");
        let serve_command = &tree
            .subcommands
            .iter()
            .find(|cmd| cmd.name == "serve")
            .expect("serve command missing");
        assert_eq!(serve_command.args.len(), 1);
        assert_eq!(serve_command.args.first().unwrap().name, "port");
    }

    #[test]
    fn parses_serve_command_executor() {
        let tree = build_command_structure(TEST_INKJETFILE).expect("build tree failed");
        let serve_command = &tree
            .subcommands
            .iter()
            .find(|cmd| cmd.name == "serve")
            .expect("serve command missing");
        assert_eq!(serve_command.script.executor, "bash");
    }

    #[test]
    fn parses_serve_command_source_with_tildes() {
        let tree = build_command_structure(TEST_INKJETFILE).expect("build tree failed");
        let serve_command = &tree
            .subcommands
            .iter()
            .find(|cmd| cmd.name == "serve")
            .expect("serve command missing");
        assert_eq!(
            serve_command.script.source,
            "echo \"Serving on port $port\"\n"
        );
    }

    #[test]
    fn parses_node_command_source_with_backticks() {
        let tree = build_command_structure(TEST_INKJETFILE).expect("build tree failed");
        let node_command = &tree
            .subcommands
            .iter()
            .find(|cmd| cmd.name == "node")
            .expect("node command missing");
        assert_eq!(
            node_command.script.source,
            "const { name } = process.env;\nconsole.log(`Hello, ${name}!`);\n"
        );
    }

    #[test]
    #[allow(clippy::indexing_slicing)]
    fn adds_verbose_named_flag_to_command_with_script() {
        let tree = build_command_structure(TEST_INKJETFILE).expect("build tree failed");
        let node_command = tree
            .subcommands
            .iter()
            .find(|cmd| cmd.name == "node")
            .expect("node command missing");

        assert_eq!(node_command.named_flags.len(), 1);
        assert_eq!(node_command.named_flags[0].name, "verbose");
        assert_eq!(
            node_command.named_flags[0].desc,
            "Sets the level of verbosity"
        );
        assert_eq!(node_command.named_flags[0].short, "v");
        assert_eq!(node_command.named_flags[0].long, "verbose");
        assert!(!node_command.named_flags[0].multiple);
        assert!(!node_command.named_flags[0].takes_value);
    }

    #[test]
    fn excludes_no_script() {
        let tree = build_command_structure(TEST_INKJETFILE).expect("build tree failed");
        let cmd = tree.subcommands.iter().find(|cmd| cmd.name == "no_script");
        if cmd.is_some() {
            panic!("docs command should not exist")
        }
    }

    #[test]
    fn fails_on_bad_flag_type() {
        let expected_err = "Invalid flag type 'invalid' Expected string | number | bool.";
        const FILE: &str = r#"
## check (arg?)
OPTIONS
- flag: -b |invalid| An invalid type
```
echo "Should not print $b"
```
"#;
        let err_str = build_command_structure(FILE).expect_err("invalid type should be Err");
        assert_eq!(err_str, expected_err);
        const FILE2: &str = r#"
## check
OPTIONS
* val
    * flag: --val
    * type: invalid
```
echo "Should not print $val"
        "#;
        let err_str2 = build_command_structure(FILE2).expect_err("invalid type should be Err");
        assert_eq!(err_str2, expected_err);
    }

    #[test]
    fn fails_on_empty_name() {
        const FILE: &str = r#"
## //alias
```
echo "Should not print"
```
"#;
        let err_str = build_command_structure(FILE).expect_err("should error on no command name");
        assert_eq!(err_str, "unexpected empty heading name");
    }

    #[test]
    fn removes_commands_without_code_blocks() {
        const FILE: &str = r#"
# MY COMMANDS

## Docs

Just some documentation.

## cmd

```bash
echo something
```

### docs two

#### docs two c
"#;
        let tree = build_command_structure(FILE).expect("failed to build tree");
        let docs_cmd = &tree.subcommands.iter().find(|cmd| cmd.name == "docs");
        if docs_cmd.is_some() {
            panic!("docs command should not exist")
        }
    }

    #[test]
    fn errors_if_merged_h1_has_spaces() {
        let contents = r#"
<!-- inkfile: /home/ubuntu/code/github.com/brandonkal/inkjet/tests/spaces/inkjet.md -->
# Tests for Spaces

inkjet_import: all

## before

```
echo "This has spaces"
```
<!-- inkfile: /home/ubuntu/code/github.com/brandonkal/inkjet/tests/spaces/merged.inkjet.md -->
# Tests for Spaces

## merged

```
echo "This has spaces"
```
```
    "#;
        let tree = build_command_structure(contents);
        tree.expect_err("Command names cannot contain spaces. Found 'tests for spaces'. Did you forget to wrap args in ()?");
    }

    #[test]
    fn preserves_arguments_order() {
        let contents = r#"
## order (one) (two) (three?)

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
    "#;
        let tree = build_command_structure(contents).expect("failed to build tree");
        let mut order_cmd = tree
            .subcommands
            .iter()
            .find(|cmd| cmd.name == "order")
            .unwrap()
            .clone();
        let mut ordered_result = String::new();
        for arg in order_cmd.args.iter_mut() {
            ordered_result.push_str(arg.name.clone().as_str())
        }
        assert_eq!(ordered_result, "onetwothree");
    }
}
