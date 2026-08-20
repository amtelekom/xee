use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    MissingPrefix(String),
    UnknownPrefix(String),
    InvalidComment(String),
    ParseError(ParseError),

    Unspecified(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}

#[derive(Debug)]
pub enum ParseError {
    Unspecified((String, super::Span)),
}

impl ParseError {
    pub fn span(&self) -> super::Span {
        match self {
            Self::Unspecified((_, s)) => *s,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ParseError {}
