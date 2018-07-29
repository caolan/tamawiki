//! Loading and rendering of Tera templates
use actix_web::dev::HttpResponseBuilder;
use actix_web::HttpResponse;
use serde::ser::Serialize;
use tera::Tera;
use http;


lazy_static! {
    static ref TERA: Tera = {
        compile_templates!("templates/**/*")
    };
}

/// Attempts to render the body of the provided response using the
/// given template name and data. If the rendering fails, the response
/// will be replaced with an internal server error response.
pub fn render_response<T: Serialize>(mut res: HttpResponseBuilder,
                                     template_name: &str,
                                     data: &T) -> HttpResponse
{
    match TERA.render(template_name, data) {
        Ok(data) => res.body(data),
        Err(_) => {
            HttpResponse::InternalServerError()
                .header(http::header::CONTENT_TYPE, "text/html")
                .body("Internal server error")
        },
    }
}
