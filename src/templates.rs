use tera;

lazy_static! {
    pub static ref TERA: tera::Tera = { compile_templates!("templates/**/*") };
}
