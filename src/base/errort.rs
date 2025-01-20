use std::error::Error;
use std::fmt;

pub type BoxResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug)]
pub struct JudgeError {
    details: String
}

impl JudgeError {
    pub fn new(msg: &str) -> Self {
        JudgeError { details: msg.to_string() }
    }
}

impl fmt::Display for JudgeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for JudgeError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[derive(Debug)]
pub struct RouteError {
    details: String
}

impl RouteError {
    pub fn new(msg: &str) -> Self {
        RouteError { details: msg.to_string() }
    }
}

impl fmt::Display for RouteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for RouteError {
    fn description(&self) -> &str {
        &self.details
    }
}