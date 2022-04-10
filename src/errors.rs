use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub struct TowError {
    msg: String,
}

impl TowError {
    pub fn new(msg: &str) -> TowError {
        TowError {
            msg: msg.to_owned(),
        }
    }
}

impl fmt::Display for TowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl error::Error for TowError {}

impl From<String> for TowError {
    fn from(e: String) -> Self {
        TowError { msg: e }
    }
}

impl From<io::Error> for TowError {
    fn from(e: io::Error) -> Self {
        TowError::new(&e.to_string())
    }
}

impl From<reqwest::Error> for TowError {
    fn from(e: reqwest::Error) -> Self {
        TowError::new(&e.to_string())
    }
}

impl From<reqwest::header::ToStrError> for TowError {
    fn from(e: reqwest::header::ToStrError) -> Self {
        TowError::new(&e.to_string())
    }
}
