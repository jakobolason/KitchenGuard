use actix::{Actor, Handler, Context, ActorFutureExt, ResponseActFuture, WrapFuture, ResponseFuture};
use ring::pbkdf2;
use std::num::NonZeroU32;
use data_encoding::HEXLOWER;
use mongodb::{bson::doc, Client,};
use log::error;
use futures_util::StreamExt;

use super::{
    cookie_manager::CookieManager, 
    shared_struct::{LoggedInformation, LoginInformation, ResIdFetcher, ValidateSession, INFO, RESIDENT_DATA, RESIDENT_LOGS, USERS}
};
use super::state_handler::Event;
pub struct WebHandler {
    cookie_manager: CookieManager,
    db_client: Client,
}

impl WebHandler {
    pub fn new(cookie_manager: CookieManager, db_client: Client) -> WebHandler {
        WebHandler { cookie_manager, db_client }
    }

    async fn check_login(username: String, passwd: String, db_client: Client) -> Result<Vec<String>, std::io::Error> {
        // checks the db for username
        let users = db_client.database(USERS).collection::<LoggedInformation>(INFO);
        match users
            .find_one(doc! {"username": &username})
            .await {
                Ok(Some(doc)) => {
                    if WebHandler::verify_password(passwd.as_str(), &doc.salt.bytes, doc.password.as_str()) {
                        Ok(doc.res_ids)
                    } else {
                        eprintln!("wrong password entered");
                        Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "wrong credentials"))
                    }
                },
                Ok(None) => {
                    eprintln!("No login information found for res_id: {}", username);
                    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "no user found"))
                }
                Err(err) => {
                    eprintln!("Error querying logins: {:?}", err);
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

    async fn get_info(res_id: &str, db_client: Client) -> Option<Vec<Event>> {
        match db_client
            .database(RESIDENT_DATA)
            .collection::<Event>(RESIDENT_LOGS)
            .find(doc! {"res_id": res_id})
            .await
        {
            Ok(mut cursor) => {
                let mut result = Vec::new();

                while let Some(doc) = cursor.next().await {
                    match doc {
                        Ok(d) => {
                        println!("Found document: {:?}", d);
                        result.push(d)
                    },
                        Err(e) => eprintln!("Error reading doc: {:?}", e),
                    }
                }
                // Check if the result is empty
                if result.is_empty() {
                    println!("No documents found for res_id: {:?}", res_id);
                }
                // Return the result
                Some(result)

            },
            Err(err) => {
                eprintln!("Error querying database: {:?}", err);
                None
            }
        }
    }
}

impl Actor for WebHandler {
    type Context = Context<Self>;
}

impl Handler<LoginInformation> for WebHandler {
    type Result = ResponseActFuture<Self, Option<String>>;

    fn handle(&mut self, info: LoginInformation, _ctx: &mut Self::Context) -> Self::Result {
        let db_client = self.db_client.clone();
        
        // Create a future that will have access to self when resolved
        Box::pin(
            async move {
                WebHandler::check_login(
                    info.username.clone(), 
                    info.password.clone(), 
                    db_client
                ).await
            }
            .into_actor(self)  // Convert the future into an actor future
            .map(|result, actor, _ctx| {
                // This closure will have access to the actor (self)
                match result {
                    Ok(vec) => {
                        Some(actor.cookie_manager.create_new_cookie(vec))
                    },
                    Err(e) => {
                        error!("Login check failed: {}", e);
                        None
                    }
                }
            })
        )
    }
}

// This is a weird workaround instead of not sending directly, but makes since architecturally
impl Handler<ValidateSession> for WebHandler {
    type Result = Option<Vec<String>>;

    fn handle(&mut self, validate_session: ValidateSession, _ctx: &mut Self::Context) -> Self::Result {
        self.cookie_manager.validate_session(validate_session.cookie)
    } 
}

// Specify the expected result from handling this message
impl Handler<ResIdFetcher> for WebHandler {
    type Result = ResponseFuture<Option<Vec<Event>>>;

    fn handle(&mut self, msg: ResIdFetcher, _ctx: &mut Self::Context) -> Self::Result {
        let db_client = self.db_client.clone();

        Box::pin(async move {
            // Use the db_client to find documents matching the res_uid
            println!("Fetching logs for res_id: {:?}", msg.res_id);
            WebHandler::get_info(&msg.res_id, db_client).await
        })
    }
}


#[cfg(test)]
mod tests {
    //use super::*;

}