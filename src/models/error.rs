use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub struct ProcessorError {
    pub error: String,
}

pub struct ErrorWithMessage {
    pub message: String,
}

impl ErrorWithMessage {
    pub fn new(message: String) -> Self {
        Self {
            message
        }
    }
}

impl Debug for ErrorWithMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Display for ErrorWithMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ErrorWithMessage {

}