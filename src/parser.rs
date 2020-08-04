#![warn(clippy::indexing_slicing)]
use pulldown_cmark::CodeBlockKind::Fenced;
use pulldown_cmark::{
    Event::{Code, End, Html, Start, Text},
    Options, Parser, Tag,
};

use crate::command::{Arg, CommandBlock, OptionFlag};

/// Creates the message that is returned on an error
fn invalid_type_msg(t: &str) -> String {
    format!("Invalid flag type '{}' Expected string | number | bool.", t)
}

/// The main parsing logic. Takes an inkfile content as a string and returns the parsed CommandBlock.
pub fn build_command_structure(inkfile_contents: &str) -> Result<CommandBlock, String> {
    let parser = create_markdown_parser(&inkfile_contents);
    let mut commands = vec![];
    let mut current_command = CommandBlock::new(1);
    let mut current_option_flag = OptionFlag::new();
    let mut text = "".to_string();
    let mut list_level = 0;
    let mut first_was_pushed = false;
    let mut current_file = "".to_string();

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
                    Tag::CodeBlock(cb) => {
                        if let Fenced(lang_code) = cb {
                            current_command.end = range.start;
                            current_command.script.executor = lang_code.to_string();
                        }
                    }
                    Tag::List(_) => {
                        // We're in an options list if the current text above it is "OPTIONS"
                        if text == "OPTIONS" || list_level > 0 {
                            list_level += 1;
                        }
                    }
                    _ => (),
                };

                // Reset all state
                text = "".to_string();
            }
            End(tag) => match tag {
                Tag::Heading(heading_level) => {
                    let (name, aliases, args) = parse_heading_to_cmd(heading_level, text.clone());
                    if name.is_empty() {
                        return Err("unexpected empty heading name".to_owned());
                    }
                    current_command.name = name;
                    current_command.args = args;
                    current_command.aliases = aliases;
                }
                Tag::BlockQuote => {
                    current_command.desc = text.clone();
                }
                Tag::CodeBlock(_) => {
                    current_command.script.source = text.to_string();
                }
                Tag::List(_) => {
                    // Don't go lower than zero (for cases where it's a non-OPTIONS list)
                    list_level = std::cmp::max(list_level - 1, 0);
                    // Must be finished parsing the current option
                    if list_level == 1 {
                        // Add the current one to the list and start a new one
                        current_command
                            .option_flags
                            .push(current_option_flag.clone());
                        current_option_flag = OptionFlag::new();
                    }
                }
                _ => (),
            },
            Text(body) => {
                text += &body.to_string();

                // Options level 1 is the flag name
                if list_level == 1 {
                    if text.contains(':') {
                        // Shorthand syntax
                        let mut flag_split = text.splitn(2, ':');
                        if flag_split.next().unwrap_or("").trim() == "flags" {
                            let val = flag_split.next().unwrap_or("").trim();
                            let mut desc_words = String::with_capacity(val.len());
                            for word in val.split_whitespace() {
                                if word.starts_with("--") {
                                    let name = word.split("--").collect::<Vec<&str>>().join("");
                                    current_option_flag.long = name.clone();
                                    current_option_flag.name = name;

                                // Must be a short flag name
                                } else if word.starts_with('-') {
                                    // Get the single char
                                    let name = word.get(1..2).unwrap_or("");
                                    current_option_flag.short = name.to_string();
                                } else if word.starts_with('|') && word.ends_with('|') {
                                    let mut kind = word.to_owned();
                                    kind.pop();
                                    kind.remove(0);
                                    match kind.as_str() {
                                        "string" => {
                                            current_option_flag.takes_value = true;
                                        }
                                        "number" => {
                                            current_option_flag.takes_value = true;
                                            current_option_flag.validate_as_number = true;
                                        }
                                        "bool" | "boolean" => {}
                                        t => {
                                            return Err(invalid_type_msg(t));
                                        }
                                    }
                                } else {
                                    desc_words.push(' ');
                                    desc_words.push_str(word)
                                }
                            }
                            current_option_flag.desc = desc_words.trim().to_string();
                        }

                        // Add the current one to the list and start a new one
                        current_command
                            .option_flags
                            .push(current_option_flag.clone());
                        current_option_flag = OptionFlag::new();
                    } else {
                        current_option_flag.name = text.clone();
                    }
                }
                // Options level 2 is the flag config
                else if list_level == 2 {
                    let mut config_split = text.splitn(2, ':');
                    let param = config_split.next().unwrap_or("").trim();
                    let val = config_split.next().unwrap_or("").trim();
                    match param {
                        "desc" => current_option_flag.desc = val.to_string(),
                        "type" => {
                            if val == "string" || val == "number" {
                                current_option_flag.takes_value = true;
                                if val == "number" {
                                    current_option_flag.validate_as_number = true;
                                }
                            } else {
                                return Err(invalid_type_msg(val));
                            }
                        }
                        // Parse out the short and long flag names
                        "flags" => {
                            let short_and_long_flags: Vec<&str> = val.splitn(2, ' ').collect();
                            for flag in short_and_long_flags {
                                // Must be a long flag name
                                if flag.starts_with("--") {
                                    let name = flag.split("--").collect::<Vec<&str>>().join("");
                                    current_option_flag.long = name;
                                }
                                // Must be a short flag name
                                else if flag.starts_with('-') {
                                    // Get the single char
                                    let name = flag.get(1..2).unwrap_or("");
                                    current_option_flag.short = name.to_string();
                                }
                            }
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
                text += &html.to_string();
            }
            Code(inline_code) => {
                text += &format!("`{}`", inline_code);
            }
            _ => (),
        };
    }

    // Add the last command
    commands.push(current_command.build());

    // Convert the flat commands array and to a tree of subcommands based on level
    let all = treeify_commands(commands);
    let all = remove_duplicates(all);
    let root_command = all.first().expect("root command must exist");

    // The command root and a possible init script
    Ok(root_command.clone())
}

// remove duplicate commands to enable override function
fn remove_duplicates(mut cmds: Vec<CommandBlock>) -> Vec<CommandBlock> {
    trait Dedup<T: PartialEq + Clone> {
        fn clear_duplicates(&mut self);
    }
    impl<T: PartialEq + Clone> Dedup<T> for Vec<T> {
        fn clear_duplicates(&mut self) {
            let mut already_seen = vec![];
            self.retain(|item| {
                if already_seen.contains(item) {
                    false
                } else {
                    already_seen.push(item.clone());
                    true
                }
            })
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
    Parser::new_ext(&inkfile_contents, options)
}

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

    command_tree
}

fn parse_heading_to_cmd(heading_level: u32, text: String) -> (String, String, Vec<Arg>) {
    // Why heading_level > 2? Because level 1 is the root command title (unused)
    // and level 2 can't be a subcommand so no need to split.
    let name = (if heading_level > 2 {
        // Takes a subcommand name like this:
        // "#### db flush postgres (required_arg_name)"
        // and returns "postgres (required_arg_name)" as the actual name
        text.split(' ')
            .collect::<Vec<&str>>()
            // Get subcommand after the parent command name
            .split_at(heading_level as usize - 2)
            .1
            .join(" ")
    } else if heading_level == 1 {
        text.split_whitespace().next().unwrap().to_owned()
    } else {
        text
    })
    .to_lowercase();

    // Find any required arguments. They look like this: (required_arg_name)
    let name_and_args: Vec<&str> = name.split(|c| c == '(' || c == ')').collect();
    let (name, args) = name_and_args.split_at(1);

    let name = name.join(" ");
    let mut name_and_alias = name.trim().splitn(2, "//");
    let name = match name_and_alias.next() {
        Some(n) => String::from(n),
        _ => "".to_string(), //@cov-ignore
    };
    let alias = match name_and_alias.next() {
        Some(a) => a.to_string(),
        _ => "".to_string(),
    };

    let mut out_args: Vec<Arg> = vec![];

    if !args.is_empty() {
        let args = args.join("");
        let args: Vec<&str> = args.split(' ').collect();
        for arg in &args {
            if arg.ends_with('?') {
                let mut arg = (*arg).to_string();
                arg.pop(); // remove `?`
                out_args.push(Arg::new(arg, false, None));
            } else if arg.contains('=') {
                let parts: Vec<&str> = arg.splitn(2, '=').collect();
                // will always have >= 2 parts
                #[allow(clippy::indexing_slicing)]
                out_args.push(Arg::new(
                    parts[0].to_string(),
                    false,
                    Some(parts[1].to_string()),
                ));
            } else {
                out_args.push(Arg::new((*arg).to_string(), true, None));
            }
        }
    }

    (name, alias, out_args)
}

#[cfg(test)]
const TEST_INKJETFILE: &str = r#"
# Document Title

This is an example inkfile for the tests below.

## serve (port)

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
- flags: -s --set |bool| Which port to serve on
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
        assert_eq!(
            boolean_command
                .option_flags
                .get(0)
                .expect("option flag not attached")
                .takes_value,
            false
        );
    }

    #[test]
    fn validates_string_and_removes_duplicate() {
        let tree = build_command_structure(
            r#"
## string

> Should be ignored
OPTIONS
- flags: -s --str |bool| A boolean
```
echo "Ignore me"
```

## string
OPTIONS
- flags: -s --str |string| A string
        "#,
        )
        .expect("build tree failed");
        let string_command = &tree
            .subcommands
            .iter()
            .find(|cmd| cmd.name == "string")
            .expect("string command missing");
        assert_eq!(string_command.name, "string");
        assert_eq!(
            string_command
                .option_flags
                .get(0)
                .expect("option flag not attached")
                .takes_value,
            true
        );
        assert_eq!(
            string_command
                .option_flags
                .get(0)
                .expect("option flag not attached")
                .validate_as_number,
            false
        );
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
    fn parses_serve_required_positional_arguments() {
        let tree = build_command_structure(TEST_INKJETFILE).expect("build tree failed");
        let serve_command = &tree
            .subcommands
            .iter()
            .find(|cmd| cmd.name == "serve")
            .expect("serve command missing");
        assert_eq!(serve_command.args.len(), 1);
        assert_eq!(serve_command.args.get(0).unwrap().name, "port");
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
    fn adds_verbose_optional_flag_to_command_with_script() {
        let tree = build_command_structure(TEST_INKJETFILE).expect("build tree failed");
        let node_command = tree
            .subcommands
            .iter()
            .find(|cmd| cmd.name == "node")
            .expect("node command missing");

        assert_eq!(node_command.option_flags.len(), 1);
        assert_eq!(node_command.option_flags[0].name, "verbose");
        assert_eq!(
            node_command.option_flags[0].desc,
            "Sets the level of verbosity"
        );
        assert_eq!(node_command.option_flags[0].short, "v");
        assert_eq!(node_command.option_flags[0].long, "verbose");
        assert_eq!(node_command.option_flags[0].multiple, false);
        assert_eq!(node_command.option_flags[0].takes_value, false);
    }

    #[test]
    fn does_not_add_verbose_optional_flag_to_command_with_no_script() {
        let tree = build_command_structure(TEST_INKJETFILE).expect("build tree failed");
        let no_script_command = tree
            .subcommands
            .iter()
            .find(|cmd| cmd.name == "no_script")
            .expect("no_script command missing");

        assert_eq!(no_script_command.option_flags.len(), 0);
    }

    #[test]
    fn fails_on_bad_flag_type() {
        let expected_err = "Invalid flag type 'invalid' Expected string | number | bool.";
        const FILE: &str = r#"
## check
OPTIONS
- flags: -b |invalid| An invalid type
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
    * flags: --val
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
}
