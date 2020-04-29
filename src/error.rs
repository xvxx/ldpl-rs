#![allow(unused_macros)]
use crate::parser::Rule;
use std::{error, fmt, io};

#[derive(Debug)]
pub struct LDPLError {
    pub details: String,
    pub line: usize,
    pub col: usize,
    pub len: usize,
}

impl LDPLError {
    pub fn new(details: String, line: usize, col: usize, len: usize) -> LDPLError {
        LDPLError {
            details,
            line,
            col,
            len,
        }
    }
}

impl error::Error for LDPLError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl fmt::Display for LDPLError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}", self.details)
    }
}

impl From<Result<String, String>> for LDPLError {
    fn from(error: Result<String, String>) -> Self {
        LDPLError {
            details: format!("{}", error.unwrap_err()),
            line: 0,
            col: 0,
            len: 1,
        }
    }
}

impl From<pest::error::Error<Rule>> for LDPLError {
    fn from(error: pest::error::Error<Rule>) -> Self {
        LDPLError {
            details: format!("{}", error),
            line: 0,
            col: 0,
            len: 1,
        }
    }
}

impl From<io::Error> for LDPLError {
    fn from(error: io::Error) -> Self {
        LDPLError {
            details: format!("{}", error),
            line: 0,
            col: 0,
            len: 1,
        }
    }
}

impl From<LDPLError> for io::Error {
    fn from(error: LDPLError) -> Self {
        io::Error::new(io::ErrorKind::Other, error.details)
    }
}

/// Parse error. Give it the token you got and what you expected.
macro_rules! parse_error {
    ($got:expr, $want:expr) => {{
        use crate::LDPLError;
        Err(LDPLError::new(format!("expected {}, got {:?}", $want, $got.kind), $got.line, $got.col, $got.len))
    }};
    ($got:expr, $want:expr, $($args:expr),+) => {
        parse_error!($got, format!($want, $($args),*));
    };
}

/// Create an error with line and col information.
macro_rules! line_error {
    ($line:expr, $col:expr, $msg:expr) => {{
        use crate::LDPLError;
        Err(LDPLError::new($msg.into(), $line, $col, 1))
    }};
    ($line:expr, $col:expr, $msg:expr, $($args:expr),+) => {
        line_error!($line, $col, format!($msg, $($args),*));
    };
}

/// Convenient way to create an Err(LDPLError{}).
macro_rules! error {
    ($msg:expr) => {
        line_error!(0, 0, $msg)
    };
    ($msg:expr, $($args:expr),*) => {
        error!(format!($msg, $($args),*));
    };
}
