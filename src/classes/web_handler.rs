use actix::{Actor, Handler, Context, ActorFutureExt, ResponseActFuture, WrapFuture, ResponseFuture};
use ring::pbkdf2;
use std::num::NonZeroU32;
use data_encoding::HEXLOWER;
use mongodb::{bson::doc, Client,};
use log::error;
use futures_util::StreamExt;

use crate::classes::shared_struct::{HealthData, SensorLookup, DEVICE_HEALTH, RESIDENT_LOGS, SENSOR_LOOKUP};

use super::{
    cookie_manager::CookieManager, 
    shared_struct::{Event, GetHealthData, GetStoveData, HealthCheck, LoginInformation, ResIdFetcher, StateLog, UsersLoggedInformation, ValidateSession, INFO, RESIDENT_DATA, STATES, USERS}
};
pub struct WebHandler {
    cookie_manager: CookieManager,
    db_client: Client,
}

impl WebHandler {
    pub fn new(cookie_manager: CookieManager, db_client: Client) -> WebHandler {
        WebHandler { cookie_manager, db_client }
    }

    async fn check_login(username: String, passwd: String, db_client: Client) -> Result<Vec<String>, std::io::ErrorKind> {
        // checks the db for username
        let users = db_client.database(USERS).collection::<UsersLoggedInformation>(INFO);
        match users
            .find_one(doc! {"username": &username})
            .await {
                Ok(Some(doc)) => {
                    if WebHandler::verify_password(passwd.as_str(), &doc.salt.bytes, doc.password.as_str()) {
                        Ok(doc.res_ids)
                    } else {
                        eprintln!("wrong password entered");
                        Err(std::io::ErrorKind::PermissionDenied)
                    }
                },
                Ok(None) => {
                    eprintln!("No login information found for res_id: {}", username);
                    Err(std::io::ErrorKind::NotFound)
                }
                Err(err) => {
                    eprintln!("Error querying logins: {:?}", err);
                    Err(std::io::ErrorKind::Other)
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

    async fn get_info<T>(res_id: &str, collection: &str, db_client: Client) -> Option<Vec<T>>
    where 
    T: std::marker::Send + Sync + serde::de::DeserializeOwned + serde::Serialize + std::fmt::Debug + 'static 
    {
        match db_client
            .database(RESIDENT_DATA)
            .collection::<T>(collection)
            .find(doc! {"res_id": res_id})
            // .sort(doc!{"_id": -1})
            .await
        {
            Ok(mut cursor) => {
                let mut result = Vec::new();

                while let Some(doc) = cursor.next().await {
                    match doc {
                        Ok(d) => {
                        // println!("Found document: {:?}", d);
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

// This is a weird workaround instead of not sending directly, but makes sense architecturally
impl Handler<ValidateSession> for WebHandler {
    type Result = Option<Vec<String>>;

    fn handle(&mut self, validate_session: ValidateSession, _ctx: &mut Self::Context) -> Self::Result {
        self.cookie_manager.validate_session(validate_session.cookie)
    } 
}

// Specify the expected result from handling this message
impl Handler<ResIdFetcher> for WebHandler {
    type Result = ResponseFuture<Option<Vec<StateLog>>>;

    fn handle(&mut self, msg: ResIdFetcher, _ctx: &mut Self::Context) -> Self::Result {
        let db_client = self.db_client.clone();

        Box::pin(async move {
            // Use the db_client to find documents matching the res_id
            println!("Fetching state logs for res_id: {:?}", msg.res_id);
            WebHandler::get_info::<StateLog>(&msg.res_id, STATES, db_client).await
        })
    }
}

impl Handler<GetStoveData> for WebHandler {
    type Result = ResponseFuture<Option<Vec<Event>>>;

    fn handle(&mut self, msg: GetStoveData, _ctx: &mut Self::Context) -> Self::Result {
        let db_client = self.db_client.clone();
        println!("Attempting to get stove data");

        Box::pin(async move {
            let res_id = msg.res_id;
            let sensors = match db_client
                .database(RESIDENT_DATA)
                .collection::<SensorLookup>(SENSOR_LOOKUP)
                .find_one(doc! {"res_id": &res_id})
                .sort(doc!{"_id": -1})
                .await {
                    Ok(Some(doc)) => doc,
                    Ok(None) => return None,
                    Err(err) => {
                        eprint!("error occured whilst looking up sensors: {:?}", err);
                        return None
                    }
                };
            // Use the db_client to find documents matching the res_id
            // match the sensor data with the power plug values
            match db_client
                .database(RESIDENT_DATA)
                .collection::<Event>(RESIDENT_LOGS)
                .find(doc! {"res_id": &res_id, "device_model": sensors.power_plug})
                .await
            {
                Ok(mut cursor) => {
                    let mut result = Vec::new();
                    while let Some(doc) = cursor.next().await {
                        match doc {
                            Ok(d) => {
                            // println!("Found document: {:?}", d);
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
        })
    }
}

impl Handler<GetHealthData> for WebHandler {
    type Result = ResponseFuture<Option<HealthCheck>>;

    fn handle(&mut self, msg: GetHealthData, _ctx: &mut Self::Context) -> Self::Result {
        let db_client = self.db_client.clone();

        Box::pin(async move {
            // Use the db_client to find documents matching the res_id
            println!("Fetching healthdata logs for res_id: {:?}", msg.res_id);
            match db_client
                .database(RESIDENT_DATA)
                .collection::<HealthCheck>(DEVICE_HEALTH)
                .find_one(doc! {"res_id": &msg.res_id})
                .sort(doc!{"_id": -1})
                .await
            {
                Ok(opt) => opt,
                Err(err) => {
                    eprintln!("An error occurred whilst getting latest health check: {:?}", err);
                    None
                } 
            }
        })
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

}