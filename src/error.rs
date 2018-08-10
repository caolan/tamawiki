use std::fmt::{self, Display};
use std::error::Error;


/// Error conditions that could not be handled as a HTTP response
#[derive(Debug)]
pub struct TamaWikiError {}

impl fmt::Display for TamaWikiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TamaWikiError")
    }
}

impl Error for TamaWikiError {}

#[derive(Debug)]
pub enum HttpError {
    InternalServerError(String),
    MethodNotAllowed,
    BadRequest,
    NotFound,
    Unauthorized,
}

impl Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HttpError::MethodNotAllowed => write!(f, "MethodNotAllowed"),
            HttpError::BadRequest => write!(f, "BadRequest"),
            HttpError::NotFound => write!(f, "NotFound"),
            HttpError::Unauthorized => write!(f, "Unauthorized"),
            HttpError::InternalServerError(ref err) => 
                write!(f, "InternalServerError: {}", err),
        }
    }
}

impl Error for HttpError {
    fn description(&self) -> &str {
        match *self {
            HttpError::MethodNotAllowed => "method not allowed",
            HttpError::BadRequest => "bad request",
            HttpError::NotFound => "not found",
            HttpError::Unauthorized => "unauthorized",
            HttpError::InternalServerError(ref err) => err,
        }
    }
}
