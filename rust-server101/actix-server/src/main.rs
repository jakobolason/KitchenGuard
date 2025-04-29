use actix_web::{
    cookie::{Key, SameSite}, error, middleware::Logger, web, App, HttpResponse, HttpServer
};
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix::Actor;
use env_logger::Env;
use actix_cors::Cors;
// use model::User;
use mongodb::Client;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/* 
    This introduces the binary to the classes.rs and routes.rs file, which includes the files under the folders 
*/
// controls endpoint logic
pub mod routes; 
// implementation of business logic
pub mod classes;

use crate::classes::{
    job_scheduler::{JobsScheduler, ScheduledTask, StartChecking},
    state_handler::{StateHandler, SetJobScheduler},
    shared_struct::AppState,
    cookie_manager::CookieManager,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log::info!("Setting up mongoDB connection...");
    // Setup mongodb connection
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());

    // let all workers use the client/Mongodb connection
    let db_client = Client::with_uri_str(uri).await.expect("failed to connect");
    // create_username_index(&client).await;
    log::info!("DB connection successfull, setting up routes...");

    // shows logging information when reaching server
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    log::info!("Setting up state and scheduler... ");
    // Start state handler actor
    let state_handler = StateHandler {
        db_client: db_client.clone(),
        job_scheduler: None,
        is_test: false,
    }.start();
    
    // Start job scheduler actor and link to state handler
    let job_scheduler = JobsScheduler {
        tasks: Arc::new(Mutex::new(VecDeque::<ScheduledTask>::new())),
        state_handler: state_handler.clone(),
    }.start();
    
    // Update state handler with job scheduler reference
    state_handler.do_send(SetJobScheduler {
        scheduler: Some(job_scheduler.clone()),
    });
    // Start the scheduler's checking of tasks overdue
    job_scheduler.do_send(StartChecking);

    let cookie_manager = CookieManager::new(12).start(); // 12 hour sessions

    let secret_key = Key::generate();

    log::info!("Finished setting up state and scheduler! Now setting AppState... ");
    // Create app state to share actor addresses
    let app_state = web::Data::new(AppState {
        state_handler: state_handler.clone(),
        job_scheduler: job_scheduler.clone(),
        cookie_manager: cookie_manager.clone(),
        db_client: db_client.clone(),
    });

    log::info!("Finished setup! Starting server... ");
    HttpServer::new(move|| {
        let _json_config = web::JsonConfig::default()
            .limit(4096)
            .error_handler(|err, _req| {
                // create custom error response
                error::InternalError::from_response(err, HttpResponse::Conflict().finish())
                    .into()
            });

        // we got cors error when connecting pi to server, so we used this
        let cors = Cors::default()
            //.allowed_origin(
            //    &(std::env::var("SERVER_URL").unwrap().to_string()+ ":" + &std::env::var("FROTEND").unwrap().to_string())
            //)
            .allow_any_origin()
            .allowed_methods(vec!["GET","POST","PUT"]);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(), secret_key.clone())
                .cookie_http_only(false)
                .cookie_same_site(SameSite::Strict)
                .build()
            )
            .app_data(app_state.clone()) // holds references to actors and db
            .configure(routes::browser::browser_config) // webhandler '/'
            .configure(routes::api::api_config)  // State handler '/api'
            // Global middleware or other configs
            .default_service(web::route().to(|| async {
                HttpResponse::NotFound().body("404 Not Found")
            }))
    })
    .bind("0.0.0.0:8080")? // change to 0.0.0.0 to expose server using computer's ip address, port 8080
    .run()
    .await
}
