use actix_web::{web, HttpResponse, http};
use actix_session::Session;
use std::fs;
use log::error;
use crate::classes::shared_struct::{AppState, LoginInformation, ValidateSession, ResUidFetcher};

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
        Err(_) => HttpResponse::InternalServerError().body("Internal server error")
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
        match app_state.web_handler.send(ValidateSession { cookie}).await {
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

async fn get_res_info(session: Session, app_state: web::Data<AppState>, res_uid_fetcher: web::Query<ResUidFetcher>) -> HttpResponse {
    println!("IN GET_RES_INFO");
    if let Some(cookie) = session.get::<String>("cookie").unwrap() {
        println!("accessed with cookie: {}", cookie);
        // check this cookie for session valid
        match app_state.web_handler.send(ValidateSession { cookie}).await {
            Ok(Some(uids)) => {


                // Check if the uid is valid
                if uids.contains(&res_uid_fetcher.res_uid) {
                    println!("Fetching resident info for UID: {}", res_uid_fetcher.res_uid);
    
                    // Extract UID before moving the Query value
                    let uid = res_uid_fetcher.res_uid.clone();
                    let resident_request = res_uid_fetcher.into_inner();
                    
                    // Fetch the resident information from the web_handler
                    let res_info = app_state.web_handler.send(resident_request).await;
                    match res_info {
                        Ok(_info) => HttpResponse::Ok().body(format!("Resident info for UID: {}", uid)),
                        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch resident information")
                    }
                

                } else {
                    HttpResponse::BadRequest().body("Invalid resident UID")
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
