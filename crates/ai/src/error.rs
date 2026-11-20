use std::fmt;

#[derive(Debug)]
pub enum AiError {
    Http(u16, String),
    Parse(String),
    Schema(String),
    Provider(String),
    Keychain(String),
    Empty,
    ModelRequired,
    UnknownProvider(String),
}

impl fmt::Display for AiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(status, body) => write!(formatter, "http {status}: {body}"),
            Self::Parse(message) => write!(formatter, "parse: {message}"),
            Self::Schema(message) => write!(formatter, "schema: {message}"),
            Self::Provider(message) => write!(formatter, "provider: {message}"),
            Self::Keychain(message) => write!(formatter, "keychain: {message}"),
            Self::Empty => write!(formatter, "empty response"),
            Self::ModelRequired => write!(formatter, "model is required"),
            Self::UnknownProvider(name) => write!(formatter, "unknown provider {name}"),
        }
    }
}

impl std::error::Error for AiError {}
