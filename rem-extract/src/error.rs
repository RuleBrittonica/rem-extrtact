use std::fmt;
use std::io;
use syn::Error as SynError;

#[derive(Debug)]
pub enum ExtractionError {
    Io(io::Error),
    Parse(SynError),
    FormatError,
    InvalidLineRange,
    InvalidColumnRange,
    InvalidCursor,
    ZeroLineIndex,
}

impl fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractionError::Io(e) => write!(f, "I/O error: {}", e),
            ExtractionError::Parse(e) => write!(f, "Parse error: {}", e),
            ExtractionError::FormatError => write!(f, "Formatting error"),
            ExtractionError::InvalidLineRange => write!(f, "Invalid line range"),
            ExtractionError::InvalidColumnRange => write!(f, "Invalid column range"),
            ExtractionError::InvalidCursor => write!(f, "Invalid cursor"),
            ExtractionError::ZeroLineIndex => write!(f, "Line index must be greater than 0. Cursor is 1-indexed."),
        }
    }
}

impl From<io::Error> for ExtractionError {
    fn from(error: io::Error) -> Self {
        ExtractionError::Io(error)
    }
}

impl From<SynError> for ExtractionError {
    fn from(error: SynError) -> Self {
        ExtractionError::Parse(error)
    }
}