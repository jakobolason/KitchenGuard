use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use mongodb::Client;

use crate::classes::shared_struct::{CreateUser, AppState, SensorLookup, InitState, Event, HealthData};

pub fn api_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/test-save", web::post().to(test_save))
            .route("/save", web::post().to(save_data))
            .route("/status", web::get().to(get_status))
            .route("/health_check", web::post().to(health_check))
            .route("/event", web::post().to(log_event))
            .route("/create_user", web::post().to(create_user))
            .route("/initialization", web::post().to(initialization))
    );
}

const DB_NAME: &str = "test";
const COLL_NAME: &str = "users";

async fn create_user(data: web::Json<CreateUser>, app_state: web::Data<AppState>) -> HttpResponse {
    println!("creating user!");
    match app_state.state_handler.send(data.into_inner()).await {
        Ok(_) => HttpResponse::Ok().body("OK"),
        Err(_) => HttpResponse::BadRequest().finish()
    }
}

/// A pi sending to this endpoint (re)starts a resident's state with 'Standby' and captures the ip address
/// If the res_id already existed in db, then it will simply overwrite it (unsafe)
async fn initialization(
    req: actix_web::HttpRequest,
    data: web::Json<SensorLookup>,
    app_state: web::Data<AppState>,
) -> HttpResponse {
    if let Some(peer_addr) = req.peer_addr() {
        println!("Initialization of pi from IP: {}", peer_addr.ip());
        app_state.state_handler.do_send(InitState { info: data.into_inner(), ip_addr: peer_addr.ip().to_string() });
        HttpResponse::Ok().body("OK")
    } else {
        println!("Could not determine the IP address of the client.");
        HttpResponse::BadRequest().body("The ip address wasn't present")
    }
}

async fn log_event(data: web::Json<Event>, app_state: web::Data<AppState>) -> HttpResponse {
    match app_state.state_handler.send(data.into_inner()).await {
        Ok(_) => HttpResponse::Ok().body("OK"),
        Err(_) => HttpResponse::BadRequest().finish()
    }
}

async fn get_status() -> HttpResponse {
    HttpResponse::Ok().body("API Status: OK")
}



async fn health_check(form: web::Json<HealthData>, app_state: web::Data<AppState>) -> HttpResponse {
    log::info!("Save endpoint reached");
    println!("{:?}", form);
    app_state.state_handler.send(form.into_inner()).await.unwrap();
    HttpResponse::Ok().body("YEP")
}

// query or body format (struct)
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
}

async fn test_save(
    form: web::Json<User>,
    client: web::Data<Client>
                    ) -> HttpResponse 
{
    let collection = client.database(DB_NAME).collection(COLL_NAME);
    let result = collection.insert_one(form.into_inner()).await;
    log::info!("Save endpoint reached");
    match result {
        Ok(_) => HttpResponse::Ok().body("user added"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

async fn save_data(
    form: web::Json<Event>,
    app_state: web::Data<AppState>
                    ) -> HttpResponse 
{
    let new_state = app_state.state_handler.send(form.into_inner()).await;
    log::info!("Save endpoint reached");
    match new_state {
        Ok(_) => HttpResponse::Ok().body("user added"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}





