use actix_web::{web, HttpResponse, http};
use std::fs;
use log::error;
use serde::{Deserialize, Serialize};
use crate::classes::{job_scheduler::JobsScheduler, state_handler::{StateHandler, Event, LoginInformation}};
use crate::classes::shared_struct::AppState;

pub fn browser_config(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(|| async { HttpResponse::Ok().body("Browser UI") }))
            .route("/dashboard", web::get().to(dashboard))
            .route("/settings", web::get().to(settings))
            .route("/login", web::post().to(login));
}


// login page, which has 2 fields, and then you submit the fields and give them as a request
async fn login(info: web::Form<LoginInformation>, app_state: web::Data<AppState>) -> HttpResponse {
    println!("username: {:?}", info.username);
    println!("passwd: {:?}", info.password);
    let password_correct = StateHandler::check_login(info.username.clone(), info.password.clone(), app_state.db_client.clone()).await;
    match password_correct {
        Ok(true) => HttpResponse::Ok().body("correct"),
        Ok(false) => HttpResponse::Unauthorized().body("incorrect"),
        Err(e) => {
            error!("Login check failed: {}", e);
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}

async fn dashboard() -> HttpResponse {
    match fs::read_to_string("./src/templates/index.html") { // files are retrived from base dir
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
