use actix::{Actor, Handler, Context, ActorFutureExt, ResponseActFuture, WrapFuture};
use ring::pbkdf2;
use std::num::NonZeroU32;
use data_encoding::HEXLOWER;
use mongodb::{bson::doc, Client,};
use log::error;

use super::{cookie_manager::CookieManager, shared_struct::{LoggedInformation, LoginInformation, ValidateSession}};

pub struct WebHandler {
    cookie_manager: CookieManager,
    db_client: Client,
}

impl WebHandler {
    pub fn new(cookie_manager: CookieManager, db_client: Client) -> WebHandler {
        WebHandler { cookie_manager, db_client }
    }

    // given a valid cookie, information from db should be fetched
    fn get_info(_res_id: String) {

    }

    pub async fn check_login(username: String, passwd: String, db_client: Client) -> Result<Vec<String>, std::io::Error> {
        // checks the db for username
        let users = db_client.database("users").collection::<LoggedInformation>("info");
        match users
            .find_one(doc! {"username": &username})
            .await {
                Ok(Some(doc)) => {
                    if WebHandler::verify_password(passwd.as_str(), &doc.salt, doc.password.as_str()) {
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

#[cfg(test)]
mod tests {
    // use super::*;

}