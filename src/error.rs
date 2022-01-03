#[derive(Debug)]
pub enum Error {
    ReadFileError(std::io::Error),
    InvalidOption(usize),
    RepeatedOption(String, usize),
    InvalidContainer(usize),
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
                Error::InvalidOption(line) =>
                    format!("Invalid option in configuration file on line {}", line),
                Error::RepeatedOption(option, line) => format!(
                    "Repeated option \"{}\" in configuration file on line {}",
                    option, line
                ),
                Error::InvalidContainer(line) =>
                    format!("Invalid container in configuration file on line {}", line),
            }
        )
    }
}
