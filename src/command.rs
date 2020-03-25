#[derive(Debug, Clone)]
pub struct Command {
    pub cmd_level: u8,
    pub name: String,
    pub alias: String,
    pub desc: String,
    pub script: Script,
    pub subcommands: Vec<Command>,
    pub args: Vec<Arg>,
    pub option_flags: Vec<OptionFlag>,
    pub start: usize,
    pub end: usize,
}

impl Command {
    pub fn new(cmd_level: u8) -> Self {
        Self {
            cmd_level,
            name: "".to_string(),
            alias: "".to_string(),
            desc: "".to_string(),
            script: Script::new(),
            subcommands: vec![],
            args: vec![],
            option_flags: vec![],
            start: 0,
            end: 0,
        }
    }

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
pub struct Script {
    // The executor to run the source with
    pub executor: String, // shell, node, ruby, python, etc...
    // The script source to execute
    pub source: String,
}

impl Script {
    pub fn new() -> Self {
        Self {
            executor: "".to_string(),
            source: "".to_string(),
        }
    }

    pub fn has_script(&self) -> bool {
        self.source != "" && self.executor != ""
    }
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: String,
    pub val: String,
    pub required: bool,
    pub default: Option<String>,
}

impl Arg {
    pub fn new(name: String, required: bool, default: Option<String>) -> Self {
        Arg {
            name,
            val: "".to_string(),
            required,
            default,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct OptionFlag {
    pub name: String,
    pub desc: String,
    pub short: String,            // v        (used as -v)
    pub long: String,             // verbose  (used as --verbose)
    pub multiple: bool,           // Can it have multiple values? (-vvv OR -i one -i two)
    pub takes_value: bool,        // Does it take a value? (-i value)
    pub validate_as_number: bool, // Should we validate it as a number?
    pub val: String,
}

impl OptionFlag {
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
