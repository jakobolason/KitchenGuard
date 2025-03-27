use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use mongodb::Client;

pub fn api_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/test-save", web::post().to(save_test))
            .route("/save", web::post().to(save_data))
            .route("/status", web::get().to(get_status))
    );
}

const DB_NAME: &str = "test";
const COLL_NAME: &str = "users";

async fn get_status() -> HttpResponse {
    HttpResponse::Ok().body("API Status: OK")
}

// async fn broker_information() -> HttpResponse {
//     // 
// }

async fn save_test(
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

// query or body format (struct)
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
}

// HEUCOD event standard, needs implementing.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Event {
    pub time_stamp: String,
    pub mode: String,
    pub event_data: String,
    pub event_type_enum: String, // Or you could define an enum here
    pub patient_id: u32,
    pub device_model: String,
    pub device_vendor: String,
    pub gateway_id: u32,
    pub id: String,
}

