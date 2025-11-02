use std::fmt::Display;

/// The different kinds of errors that can occur during argument parsing.
#[derive(Debug)]
pub enum ArgsError {
    /// A required argument was missing.
    MissingArgument { name: String },

    /// The parser failed to parse an argument's value.
    CannotParseArgument { name: String },
}

impl ArgsError {
    pub fn missing_argument(name: &str) -> Self {
        ArgsError::MissingArgument { name: name.to_owned() }
    }

    pub fn cannot_parse_argument(name: &str) -> Self {
        ArgsError::CannotParseArgument { name: name.to_owned() }
    }
}

impl Display for ArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgsError::MissingArgument { name } => write!(f, "missing argument '{}'", name),
            ArgsError::CannotParseArgument { name } => write!(f, "cannot parse value for argument '{}'", name),
        }
    }
}

impl<T> Into<Result<T, ArgsError>> for ArgsError {
    fn into(self) -> Result<T, ArgsError> {
        Err(self)
    }
}
