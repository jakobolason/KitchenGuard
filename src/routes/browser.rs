use actix_web::{web, HttpResponse, http};
use actix_session::Session;
use std::{collections::HashMap, fs};
use log::error;
use serde_json;
use crate::classes::{shared_struct::{AppState, LoginInformation, ResIdFetcher, ValidateSession}, state_handler::StateLog};

pub fn browser_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(front_page))
        .route("/index", web::get().to(front_page))
        .route("/dashboard", web::get().to(dashboard))
        .route("/settings", web::get().to(settings))
        .route("/get_res_info", web::get().to(get_res_info))
        .route("/login", web::post().to(login));
}


// login page, which has 2 fields, and then you submit the fields and give them as a request
async fn login(info: web::Form<LoginInformation>, app_state: web::Data<AppState>, session: Session) -> HttpResponse {
    println!("username: {:?}", info.username);
    println!("passwd: {:?}", info.password);
    match app_state.web_handler.send(info.into_inner()).await {
        Ok(Some(cookie)) => {
            // sets proper headers, such that user gets the new cookie and goes to '/dashboard'
            // Store the cookie in the session
            if let Err(e) = session.insert("cookie", cookie) {
                error!("Failed to insert cookie into session: {}", e);
                return HttpResponse::InternalServerError().body("Failed to create session");
            }
            // Redirect to dashboard
            HttpResponse::SeeOther()
                .append_header(("Location", "/dashboard"))
                .body("Login successful")
        }
        Ok(None) => HttpResponse::BadRequest().into(),
        Err(_) => HttpResponse::InternalServerError().body("Internal server error")
    }
}

async fn front_page() -> HttpResponse {
    match fs::read_to_string("./src/templates/forside.html") { // files are retrived from base dir
        Ok(contents) => {
            HttpResponse::Ok()
                .content_type(http::header::ContentType::html())
                .body(contents)
        },
        Err(e) => {
            error!("Failed to read frontpage template: {}", e);
            HttpResponse::InternalServerError()
                .body("Error reading frontpage template")
        }
    }
}

async fn dashboard(session: Session, app_state: web::Data<AppState>) -> HttpResponse {
    println!("IN DASHBOARD, session:");
    // deprecated once middleware is setup
    if let Some(cookie) = session.get::<String>("cookie").unwrap() {
        println!("accessed with cookie: {}", cookie);
        // check this cookie for session valid
        match app_state.web_handler.send(ValidateSession { cookie}).await {
            Ok(Some(ids)) => {
                match fs::read_to_string("./src/templates/stats.html") { // files are retrived from base dir
                    Ok(contents) => {
                        HttpResponse::Ok()
                            .content_type(http::header::ContentType::html())
                            .body(contents)
                    },
                    Err(e) => {
                        error!("Failed to read frontpage template: {}", e);
                        HttpResponse::InternalServerError()
                            .body("Error reading frontpage template")
                    }
                }
            },
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

async fn get_res_info(session: Session, app_state: web::Data<AppState>) -> HttpResponse {
    println!("IN GET_RES_INFO");
    if let Some(cookie) = session.get::<String>("cookie").unwrap() {
        println!("accessed with cookie: {}", cookie);
        // check this cookie for session valid
        match app_state.web_handler.send(ValidateSession { cookie}).await {
            Ok(Some(ids)) => {
                let mut id_vals = HashMap::<String, Vec::<StateLog>>::new();
                for id in ids  {
                    println!("Fetching resident info for id: {}", id);
                    // Fetch the resident information from the web_handler
                    let res_info = app_state.web_handler.send(ResIdFetcher{res_id: id.clone()}).await;
                    match res_info {
                        Ok(Some(info)) =>id_vals.insert(id, info),
                        _ => {
                            println!("not found any info on res_id");
                            None
                        },
                    };
                }
                match serde_json::to_string(&id_vals) {
                    Ok(json) => HttpResponse::Ok().content_type(http::header::ContentType::json()).body(json),
                    Err(e) => {
                        error!("Failed to serialize id_vals: {}", e);
                        HttpResponse::InternalServerError().body("Failed to serialize response")
                    }
                }
            },
            Ok(_) => {
                HttpResponse::InternalServerError().into()
            }
            Err(_) => {
                HttpResponse::InternalServerError().into()
            }
        }
    } else {
        println!("no cookie found..");
        // User is not logged in, redirect to login
        HttpResponse::SeeOther().append_header(("Location", "/index")).finish()
    }
}
