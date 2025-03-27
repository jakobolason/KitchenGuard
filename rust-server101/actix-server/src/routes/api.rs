use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use mongodb::Client;

pub fn api_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
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

async fn save_data(
    form: web::Json<User>,
    client: web::Data<Client>
                    ) -> HttpResponse 
{
    let collection = client.database(DB_NAME).collection(COLL_NAME);
    let result = collection.insert_one(form.into_inner()).await;
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
