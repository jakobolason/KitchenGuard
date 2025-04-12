use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use mongodb::Client;

pub fn api_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/test-save", web::post().to(test_save))
            .route("/save", web::post().to(save_data))
            .route("/status", web::get().to(get_status))
            .route("/health_check", web::post().to(health_check))
    );
}

const DB_NAME: &str = "test";
const COLL_NAME: &str = "users";



async fn get_status() -> HttpResponse {
    HttpResponse::Ok().body("API Status: OK")
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct HealthData {
    pub PIR: String,
    pub LED: String,
    pub PowerPlug: String,
    pub Bridge: String,
}

async fn health_check(form: web::Json<HealthData>) -> HttpResponse {
    log::info!("Save endpoint reached");
    println!("{:?}", form);
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
    client: web::Data<Client>
                    ) -> HttpResponse 
{
    let collection = client.database(DB_NAME).collection("EventsTest");
    let result = collection.insert_one(form.into_inner()).await;
    log::info!("Save endpoint reached");
    match result {
        Ok(_) => HttpResponse::Ok().body("user added"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}


// HEUCOD event standard, needs implementing.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Event {
    pub time_stamp: String,
    pub mode: String,
    pub event_data: String,
    pub event_type_enum: String, // Or we could define an enum here
    pub res_id: u32, // changed from patient_id to res_id
    pub device_model: String,
    pub device_vendor: String,
    pub gateway_id: u32,
    pub id: String,
}

