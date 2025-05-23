use actix_web::{web, HttpResponse};

use crate::classes::shared_struct::{AddRelative, AppState, CreateUser, Event, HealthCheck, InitState, SensorLookup};

pub fn api_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/save", web::post().to(save_data))
            .route("/status", web::get().to(get_status))
            .route("/health_check", web::post().to(health_check))
            .route("/create_user", web::post().to(create_user))
            .route("add_res_to_user", web::post().to(add_res_to_user))
            .route("/initialization", web::post().to(initialization))
    );
}
async fn create_user(data: web::Form<CreateUser>, app_state: web::Data<AppState>) -> HttpResponse {
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

async fn add_res_to_user(data: web::Json<AddRelative>, app_state: web::Data<AppState>) -> HttpResponse {
    match app_state.state_handler.send(data.into_inner()).await {
        Ok(_) => HttpResponse::Ok().body("OK"),
        Err(_) => HttpResponse::BadRequest().body("something went wrong")
    }
}

async fn get_status() -> HttpResponse {
    HttpResponse::Ok().body("API Status: OK")
}

async fn health_check(form: web::Json<HealthCheck>, app_state: web::Data<AppState>) -> HttpResponse {
    log::info!("Health chech endpoint reached");
    app_state.state_handler.send(form.into_inner()).await.unwrap();
    HttpResponse::Ok().body("YEP")
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





