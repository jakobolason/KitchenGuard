use actix_web::{
    body::MessageBody, cookie::{time::format_description::well_known, Key}, dev::{ServiceRequest, ServiceResponse}, error, middleware::{from_fn, Logger, Next}, web, App, Error, HttpResponse, HttpServer
};
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix::Actor;
use env_logger::Env;
use actix_cors::Cors;
// use model::User;
use mongodb::Client;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

// controls endpoint logic
mod routes {
    pub mod api;
    pub mod browser;
}
// implementation of business logic
mod classes {
    pub mod job_scheduler;
    pub mod state_handler;
    pub mod shared_struct;
    pub mod cookie_manager;
}
use crate::classes::{
    job_scheduler::{JobsScheduler, ScheduledTask, StartChecking},
    state_handler::{StateHandler, SetJobScheduler},
    shared_struct::AppState,
    cookie_manager::CookieManager,
};



async fn my_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    // pre-processing
    println!("req: {:?}", req);
    next.call(req).await
    // post-processing
}

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
            .wrap(from_fn(my_middleware))
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
