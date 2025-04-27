use actix_web::{web, HttpResponse, http};
use actix_session::Session;
use std::fs;
use log::error;
use actix_web::{body::MessageBody, middleware::Next, dev::{ServiceRequest, ServiceResponse}, Error};
use crate::classes::{
    shared_struct::{AppState, LoginInformation, LoggedInformation},
    // web_handler::WebHandler,
};

use ring::{digest, pbkdf2};
use std::num::NonZeroU32;
use data_encoding::HEXLOWER;
use mongodb::{bson::doc, Client,};

pub struct WebHandler {
    
}

impl WebHandler {
    // given a valid cookie, html with information from state server
    // should be provided
    pub fn get_info(cookie: String) {

    }

    pub async fn check_login(username: String, passwd: String, db_client: Client) -> Result<bool, std::io::Error> {
        // checks the db for username
        let users = db_client.database("users").collection::<LoggedInformation>("info");
        match users
            .find_one(doc! {"username": &username})
            .await {
                Ok(Some(doc)) => {
                    if WebHandler::verify_password(passwd.as_str(), &doc.salt, doc.password.as_str()) {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                },
                Ok(None) => {
                    eprintln!("No sensors found for res_id: {}", username);
                    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "no user found"))
                }
                Err(err) => {
                    eprintln!("Error querying sensors: {:?}", err);
                    Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Database error: {:?}", err)))
                }
            }
    }

    fn verify_password(password: &str, salt: &[u8], stored_hash_hex: &str) -> bool {
        // Convert hex string back to bytes
        let stored_hash = HEXLOWER.decode(stored_hash_hex.as_bytes()).unwrap();
        
        // Hash the input password with the same parameters
        let n_iter = NonZeroU32::new(100_000).unwrap();
        let alg = pbkdf2::PBKDF2_HMAC_SHA256;
        
        pbkdf2::verify(
            alg,
            n_iter,
            salt,
            password.as_bytes(),
            &stored_hash,
        ).is_ok()
    }

    pub fn hash_password(password: &str, salt: &[u8]) -> String {
        // Configure PBKDF2 parameters
        let n_iter = NonZeroU32::new(100_000).unwrap();
        let alg = pbkdf2::PBKDF2_HMAC_SHA256;
        
        // Output buffer for the hash
        let mut hash = [0u8; digest::SHA256_OUTPUT_LEN];
        
        pbkdf2::derive(
            alg,
            n_iter,
            salt,
            password.as_bytes(),
            &mut hash,
        );
        
        // Convert to hex string
        HEXLOWER.encode(&hash)
    }

    
}

pub fn browser_config(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(front_page))
        .route("/index", web::get().to(front_page))
        .route("/dashboard", web::get().to(dashboard))
        .route("/settings", web::get().to(settings))
        .route("/login", web::post().to(login));
}


// login page, which has 2 fields, and then you submit the fields and give them as a request
async fn login(info: web::Form<LoginInformation>, app_state: web::Data<AppState>) -> HttpResponse {
    println!("username: {:?}", info.username);
    println!("passwd: {:?}", info.password);
    let password_correct = WebHandler::check_login(info.username.clone(), info.password.clone(), app_state.db_client.clone()).await;
    match password_correct {
        Ok(true) => HttpResponse::Ok().body("correct"),
        Ok(false) => HttpResponse::Unauthorized().body("incorrect"),
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

async fn dashboard(session: Session) -> HttpResponse {
    // deprecated once middleware is setup
    if let Some(user_id) = session.get::<i32>("user_id").unwrap() {
        let username: String = session.get("username").unwrap().unwrap();
        // User is logged in
        HttpResponse::Ok().body(format!("Welcome to your dashboard, {}!", username))
    } else {
        // User is not logged in, redirect to login
        HttpResponse::SeeOther().append_header(("Location", "")).finish()
    }
}

async fn settings() -> HttpResponse {
    HttpResponse::Ok().body("Settings Page")
}
