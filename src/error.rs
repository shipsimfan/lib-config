#[derive(Debug)]
pub enum Error {
    ReadFileError(std::io::Error),
    RepeatedOption(String, usize),
    UnexpectedComment(usize),
    UnexpectedEndOfLine(usize),
    UnexpectedEndOfFile,
    UnexpectedEndOfContainer(usize),
    UnexpectedCharacter(char, usize),
    EmptyKey(usize),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::ReadFileError(error) =>
                    format!("Unable to read configuration file ({})", error),
                Error::RepeatedOption(option, line) => format!(
                    "Repeated option \"{}\" in configuration file on line {}",
                    option, line
                ),
                Error::UnexpectedComment(line) => format!("Unexpected comment on line {}", line),
                Error::UnexpectedEndOfLine(line) =>
                    format!("Unexpected end of line on line {}", line),
                Error::UnexpectedEndOfFile => format!("Unexpected end of file"),
                Error::UnexpectedEndOfContainer(line) =>
                    format!("Unexpected end of container on line {}", line),
                Error::EmptyKey(line) => format!("Empty key on line {}", line),
                Error::UnexpectedCharacter(character, line) =>
                    format!("Unexpected character '{}' on line {}", character, line),
            }
        )
    }
}
