use actix_web::dev::HttpResponseBuilder;
use actix_web::HttpResponse;
use serde::ser::Serialize;
use tera::Tera;
use http;


lazy_static! {
    static ref TERA: Tera = {
        let tera = compile_templates!("templates/**/*");
        // and we can add more things to our instance if we want to
        // tera.autoescape_on(vec!["html", ".sql"]);
        // tera.register_filter("do_nothing", do_nothing_filter);
        tera
    };
}

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
