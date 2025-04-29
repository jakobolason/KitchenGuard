use actix_web::{web, HttpResponse, http};
use actix_session::Session;
use std::fs;
use log::error;
use actix_web::{body::MessageBody, middleware::Next, dev::{ServiceRequest, ServiceResponse}, Error};
use crate::classes::{
    cookie_manager::{CreateNewCookie, ValidateSession}, shared_struct::{AppState, LoggedInformation, LoginInformation},
    web_handler::WebHandler,
};


pub fn browser_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(front_page))
        .route("/index", web::get().to(front_page))
        .route("/dashboard", web::get().to(dashboard))
        .route("/settings", web::get().to(settings))
        .route("/login", web::post().to(login));
}


// login page, which has 2 fields, and then you submit the fields and give them as a request
async fn login(info: web::Form<LoginInformation>, app_state: web::Data<AppState>, session: Session) -> HttpResponse {
    println!("username: {:?}", info.username);
    println!("passwd: {:?}", info.password);
    let list_of_uids = WebHandler::check_login(info.username.clone(), info.password.clone(), app_state.db_client.clone()).await;
    match list_of_uids {
        Ok(vec) => {
            match app_state.cookie_manager.send(CreateNewCookie {res_uids: vec}).await {
                Ok(cookie) => {
                    // sets proper headers, such that user gets the new cookie and goes to '/dashboard'
                    // Store the cookie in the session
                    if let Err(e) = session.insert("cookie", cookie.clone()) {
                        error!("Failed to insert cookie into session: {}", e);
                        return HttpResponse::InternalServerError().body("Failed to create session");
                    }

                    // Redirect to dashboard
                    HttpResponse::SeeOther()
                        .append_header(("Location", "/dashboard"))
                        .body("Login successful")
                }
                Err(e) => {
                    error!("Failed to create cookie: {}", e);
                    HttpResponse::InternalServerError().body("Failed to create session cookie")
                }
            }
        },
        Err(e) => {
            error!("Login check failed: {}", e);
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}

async fn front_page() -> HttpResponse {
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

async fn dashboard(session: Session, app_state: web::Data<AppState>) -> HttpResponse {
    println!("IN DASHBOARD, session:");
    // deprecated once middleware is setup
    if let Some(cookie) = session.get::<String>("cookie").unwrap() {
        println!("accessed with cookie: {}", cookie);
        // check this cookie for session valid
        match app_state.cookie_manager.send(ValidateSession { token: cookie}).await {
            Ok(Some(uids)) => HttpResponse::Ok().body(format!("welcome to your dashboard, you can use these: {:?}", uids)),
            Ok(None) => HttpResponse::ServiceUnavailable().into(),
            Err(_) => HttpResponse::BadRequest().body("You are not allowed here")
        }
    } else {
        println!("no cookie found..");
        // User is not logged in, redirect to login
        HttpResponse::SeeOther().append_header(("Location", "/index")).finish()
    }
}

async fn settings() -> HttpResponse {
    HttpResponse::Ok().body("Settings Page")
}
