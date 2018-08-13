use http::{Response, StatusCode};
use hyper::Body;
use serde_json;
use std::fmt::{self, Display};
use std::error::Error;
use tera;

use templates::TERA;


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

impl HttpError {
    fn status_code(&self) -> StatusCode {
        use self::HttpError::*;
        match *self {
            MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            BadRequest => StatusCode::BAD_REQUEST,
            NotFound => StatusCode::NOT_FOUND,
            Unauthorized => StatusCode::UNAUTHORIZED,
            InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn default_context(&self) -> serde_json::Value {
        use self::HttpError::*;
        match *self {
            InternalServerError(ref err) =>
                json!({
                    "title": "Internal Server Error",
                    "error": err
                }),
            _ =>
                json!({
                    "title": format!("{}", self)
                })
        }
    }
    
    fn default_template(&self) -> String {
        format!("{}.html", self.status_code().as_u16())
    }

    fn render_html(&self) -> tera::Result<String> {
        TERA.render(
            &self.default_template(),
            &self.default_context()
        )
    }
}

impl Into<Response<Body>> for HttpError {
    fn into(self) -> Response<Body> {
        match self.render_html() {
            Ok(html) => {
                Response::builder()
                    .status(self.status_code())
                    .body(Body::from(html))
                    .unwrap()
            },
            Err(err) => {
                for e in err.iter() {
                    eprintln!("{}", e);
                }
                if let HttpError::InternalServerError(_) = self {
                    // already failed to render a 500 error, use
                    // fallback rendering
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Internal Server Error\n"))
                        .unwrap()
                    
                } else {
                    // convert to a 500 error with the template error
                    HttpError::InternalServerError(
                        format!("Template error: {}", err)
                    ).into()
                }
            }
        }
    }
}

impl Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::HttpError::*;
        match *self {
            MethodNotAllowed => write!(f, "Method Not Allowed"),
            BadRequest => write!(f, "Bad Request"),
            NotFound => write!(f, "Not Found"),
            Unauthorized => write!(f, "Unauthorized"),
            InternalServerError(_) => write!(f, "Internal Server Error"),
        }
    }
}

impl Error for HttpError {
    fn description(&self) -> &str {
        use self::HttpError::*;
        match *self {
            MethodNotAllowed => "method not allowed",
            BadRequest => "bad request",
            NotFound => "not found",
            Unauthorized => "unauthorized",
            InternalServerError(ref err) => err,
        }
    }
}
