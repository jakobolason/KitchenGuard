use actix_web::{web, HttpResponse, http};
use actix_session::Session;
use serde::Deserialize;
use std::{collections::HashMap, fs};
use log::error;
use serde_json;
use crate::classes::shared_struct::{AppState, Event, GetHealthData, GetStoveData, HealthData, LoginInformation, ResIdFetcher, StateLog, ValidateSession};

pub fn browser_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(front_page))
        .route("/index", web::get().to(front_page))
        .route("/dashboard", web::get().to(dashboard))
        .route("/settings", web::get().to(settings))
        .route("/get_res_info", web::get().to(get_res_info))
        .route("/get_res_stove_data", web::get().to(get_res_stove_data))
        .route("/get_res_healthcheck", web::get().to(get_res_healthcheck))
        .route("/restart_alarm", web::put().to(restart_alarmed_state))
        .route("/login", web::post().to(login));
        // .service(actix_files::Files::new("/", "./src/templates").prefer_utf8(true));
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
    if let Ok(Some(cookie)) = session.get::<String>("cookie") {
        println!("accessed with cookie: {}", cookie);
        // check this cookie for session valid
        match app_state.web_handler.send(ValidateSession { cookie}).await {
            Ok(Some(_ids)) => {
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

#[derive(Deserialize)]
struct IdQuery {
    id: String,
}
async fn get_res_stove_data(session: Session, app_state: web::Data<AppState>, id: web::Query<IdQuery>) -> HttpResponse {
    println!("IN GET STOVE DATA");
    if let Ok(Some(cookie)) = session.get::<String>("cookie") {
        println!("accessed with cookie: {}", cookie);
        // check this cookie for session valid
        match app_state.web_handler.send(ValidateSession { cookie }).await {
            Ok(Some(ids)) => {
                let res_id = &id.id;
                // if user doesn't have access to requested data
                if !ids.contains(&res_id) {
                    return HttpResponse::SeeOther().append_header(("Location", "/index")).finish()
                }
                println!("Fetching resident info for id: {}", res_id);
                // Fetch the resident information from the web_handler
                let res_info = match app_state.web_handler.send(GetStoveData { res_id: res_id.to_string() }).await
                {
                    Ok(Some(logs)) => logs,
                    _ =>  Vec::<Event>::new(),
                };
                match serde_json::to_string(&res_info) {
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

async fn get_res_healthcheck(session: Session, app_state: web::Data<AppState>, id: web::Query<String>) -> HttpResponse {
    println!("IN GET HEALTH");
    if let Ok(Some(cookie))= session.get::<String>("cookie") {
        println!("accessed with cookie: {}", cookie);
        // check this cookie for session valid
        match app_state.web_handler.send(ValidateSession { cookie}).await {
            Ok(Some(ids)) => {
                let res_id = id.into_inner();
                // if user doesn't have access to requested data
                if !ids.contains(&res_id) {
                    return HttpResponse::SeeOther().append_header(("Location", "/index")).finish()
                }
                println!("Fetching resident info for id: {}", res_id);
                // Fetch the resident information from the web_handler
                let res_info = match app_state.web_handler.send(GetHealthData { res_id}).await
                {
                    Ok(Some(logs)) => logs,
                    _ => Vec::<HealthData>::new(),
                };
                match serde_json::to_string(&res_info) {
                    Ok(json) => HttpResponse::Ok().content_type(http::header::ContentType::json()).body(json),
                    Err(e) => {
                        error!("Failed to serialize res_info: {}", e);
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

async fn get_res_info(session: Session, app_state: web::Data<AppState>) -> HttpResponse {
    println!("IN GET_RES_INFO");
    if let Ok(Some(cookie)) = session.get::<String>("cookie") {
        println!("accessed with cookie: {}", cookie);
        // check this cookie for session valid
        match app_state.web_handler.send(ValidateSession { cookie}).await {
            Ok(Some(ids)) => {
                let mut id_vals: HashMap<String, Vec::<StateLog>> = HashMap::<String, Vec::<StateLog>>::new();
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

// Button that allows the user to remove the alarm
async fn restart_alarmed_state(session: Session, app_state: web::Data<AppState>, id: web::Query<String>) -> HttpResponse {
    println!("IN GET_RES_INFO");
    if let Ok(Some(cookie)) = session.get::<String>("cookie") {
        println!("accessed with cookie: {}", cookie);
        // check this cookie for session valid
        match app_state.web_handler.send(ValidateSession { cookie }).await {
            Ok(Some(ids)) => {
                let res_id = id.into_inner();
                if !ids.contains(&res_id) {
                    return HttpResponse::BadRequest().body("You do not have access to this resident!")
                }
                // Give StateHandler a message that it should return to unattended and restart the alarm
                let restart_event = Event {
                    time_stamp: chrono::Utc::now().to_string(),
                    mode: "ALARM_OFF".to_string(),
                    event_data: "user from website turns off alarm".to_string(),
                    event_type_enum: "userhandle".to_string(),
                    res_id: res_id.clone(),
                    device_model: "USER".to_string(),
                    device_vendor: "KitchenGuard".to_string(),
                    gateway_id: 1,
                    id: "1".to_string(),
                };
                match app_state.state_handler.send(restart_event).await {
                    Ok(Ok(state)) => HttpResponse::Ok().body(format!("new body: {:?}", state)),
                    Ok(Err(err)) => {
                        eprintln!("error occurred whilst sending event: {:?}", err);
                        HttpResponse::InternalServerError().into()
                    }Err(err) => {
                        eprintln!("error occurred whilst sending event: {:?}", err);
                        HttpResponse::InternalServerError().into()
                    }
                }
            },
            Ok(None) => {
                eprintln!("no cookie found");
                HttpResponse::BadRequest().body("No cookie found")
            },
            Err(err) => {
                eprintln!("an error occured whilst looking for cookie: {:?}", err);
                HttpResponse::InternalServerError().into()
            }
        }
    } else {
        println!("no cookie found..");
        // User is not logged in, redirect to login
        HttpResponse::SeeOther().append_header(("Location", "/index")).finish()
    }
}
