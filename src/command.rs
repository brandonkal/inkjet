/// CommandBlock represents a target constructed from the inkjet file parsing process.
/// It provides all the options required to then execute the target.
#[derive(Debug, Clone)]
pub struct CommandBlock {
    /// cmd_level represents the depth of a command. Subcommands have a higher cmd_level.
    pub cmd_level: u8,
    /// name is the name of this CommandBlock
    pub name: String,
    /// aliases represent alternative ways to call the given command.
    pub aliases: String,
    /// desc defines a description of the CommandBlock. It is displayed in the CLI help text.
    pub desc: String,
    /// script holds the contents of the code block and its executor (language code).
    pub script: Script,
    /// subcommands represents all the direct children of this command.
    pub subcommands: Vec<CommandBlock>,
    /// args represents positional args for this command.
    pub args: Vec<Arg>,
    /// option_flags contains a collection of all option flags that should exist for this command.
    pub option_flags: Vec<OptionFlag>,
    /// start represents the start location of this CommandBlock in the source markdown document.
    pub start: usize,
    /// end represents the end location of this CommandBlock in the source markdown document.
    pub end: usize,
    /// inkjet_file holds the filepath of the source where this CommandBlock was created.
    /// It is typically an empty string but can contain a value if this CommandBlock was imported.
    /// from this value, the working directory is derived if required.
    pub inkjet_file: String,
    /// validation_error_msg is typically empty. When it contains a value, it typically means that the user tried to provide
    /// an incorrect type to an option flag.
    pub validation_error_msg: String,
}

impl PartialEq for CommandBlock {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.cmd_level == other.cmd_level
    }
}

impl CommandBlock {
    #[must_use]
    /// Create a new CommandBlock for the given level
    pub fn new(cmd_level: u8) -> Self {
        Self {
            cmd_level,
            name: "".to_string(),
            aliases: "".to_string(),
            desc: "".to_string(),
            script: Script::new(),
            subcommands: vec![],
            args: vec![],
            option_flags: vec![],
            start: 0,
            end: 0,
            inkjet_file: "".to_string(),
            validation_error_msg: "".to_string(),
        }
    }
    #[must_use]
    /// call build to add the default verbose flag to this CommandBlock's option_flags
    pub fn build(mut self) -> Self {
        // Auto add common flags like verbose for commands that have a script source
        if !self.script.source.is_empty() {
            self.option_flags.push(OptionFlag {
                name: "verbose".to_string(),
                desc: "Sets the level of verbosity".to_string(),
                short: "v".to_string(),
                long: "verbose".to_string(),
                multiple: false,
                takes_value: false,
                validate_as_number: false,
                val: "".to_string(),
            });
        }
        self
    }
}

#[derive(Debug, Clone, Default)]
/// Script represents the source for a CommandBlock
pub struct Script {
    /// The executor to run the source with i.e. sh, node, ruby, python, etc...
    pub executor: String,
    /// The script source to execute
    pub source: String,
}

impl Script {
    #[must_use]
    /// Create a new empty script
    pub fn new() -> Self {
        Self {
            executor: "".to_string(),
            source: "".to_string(),
        }
    }
    /// Returns true if the script is non-empty
    pub fn has_script(&self) -> bool {
        !self.source.is_empty()
    }
}

#[derive(Debug, Clone)]
/// Arg represents an intermediate representation of a positional arg.
pub struct Arg {
    /// The name of the Arg
    pub name: String,
    /// The value of the Arg. This is an empty string when parsed and populated after matches are applied
    pub val: String,
    /// If a required arg is not supplied, the CLI will exit with an error.
    pub required: bool,
    /// If this Arg has a default value, we keep track of it here.
    pub default: Option<String>,
    /// Whether or not this Arg can be supplied multiple times. Values will be collected into a space-separated string.
    pub multiple: bool,
}

impl Arg {
    #[must_use]
    /// Arg::new create a new Arg
    pub fn new(name: String, required: bool, default: Option<String>, multiple: bool) -> Self {
        Arg {
            name,
            val: "".to_string(),
            required,
            default,
            multiple,
        }
    }
}

#[derive(Debug, Clone, Default)]
/// OptionFlag is an intermediate representation of an optional flag
pub struct OptionFlag {
    /// The name of the flag.
    /// This determines under what environment variable name the flag value will be exposed to a script target.
    pub name: String,
    /// desc is a text description of a flag and is displayed in help text.
    pub desc: String,
    /// The shorthand flag name. Example: v (used as -v)
    pub short: String,
    /// The longhand flag name. Example: verbose (used as --verbose)
    pub long: String,
    /// Can it have multiple values? (-vvv OR -i one -i two). This is always false by default.
    pub multiple: bool,
    /// Does the flag take a value? (-i value). Boolean flags do not take a value.
    pub takes_value: bool,
    /// Set to true if we should validate the flag as a number. Must be set with takes_value=true.
    pub validate_as_number: bool,
    /// The value of the flag. Is empty after parsing a markdown document. This value is populated when applying matches.
    pub val: String,
}

impl OptionFlag {
    #[must_use]
    /// Create a new OptionFlag
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            desc: "".to_string(),
            short: "".to_string(),
            long: "".to_string(),
            multiple: false,
            takes_value: false,
            validate_as_number: false,
            val: "".to_string(),
        }
    }
}
