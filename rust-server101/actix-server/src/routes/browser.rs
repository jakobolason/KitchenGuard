use actix_web::{web, HttpResponse, http};
use std::fs;
use log::error;

pub fn browser_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/")
            .route("", web::get().to(|| async { HttpResponse::Ok().body("Browser UI") }))
            .route("/dashboard", web::get().to(dashboard))
            .route("/settings", web::get().to(settings))
    );
}

async fn dashboard() -> HttpResponse {
    match fs::read_to_string("src/templates/index.html") { // files are retrived from base dir
        Ok(contents) => {
            HttpResponse::Ok()
                .content_type(http::header::ContentType::html())
                .body(contents)
        },
        Err(e) => {
            error!("Failed to read dashboard template: {}", e);
            HttpResponse::InternalServerError()
                .body("Error reading dashboard template")
        }
    }
}

async fn settings() -> HttpResponse {
    HttpResponse::Ok().body("Settings Page")
}
