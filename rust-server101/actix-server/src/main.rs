use actix_web::{error, web, App, HttpServer, HttpResponse};
use actix_cors::Cors;
// use model::User;
use mongodb::Client;

mod routes {
    pub mod api;
    pub mod browser;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("Setting up mongoDB connection...");
    // Setup mongodb connection
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());

    let client = Client::with_uri_str(uri).await.expect("failed to connect");
    // create_username_index(&client).await;
    log::info!("DB connection successfull, setting up routes...");
    HttpServer::new(move|| {
        let _json_config = web::JsonConfig::default()
            .limit(4096)
            .error_handler(|err, _req| {
                // create custom error response
                error::InternalError::from_response(err, HttpResponse::Conflict().finish())
                    .into()
            });
        let cors = Cors::default()
            //.allowed_origin(
            //    &(std::env::var("SERVER_URL").unwrap().to_string()+ ":" + &std::env::var("FROTEND").unwrap().to_string())
            //)
            .allow_any_origin()
            .allowed_methods(vec!["GET","POST","PUT"]);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(client.clone()))
            .configure(routes::browser::browser_config) // HTTP server '/'
            .configure(routes::api::api_config)  // State server '/api'
            // Global middleware or other configs
            .default_service(web::route().to(|| async {
                HttpResponse::NotFound().body("Not Found")
            }))
    })
    .bind("0.0.0.0:8080")? // change to 0.0.0.0 to expose server using computer's ip address, port 8080
    .run()
    .await
}
