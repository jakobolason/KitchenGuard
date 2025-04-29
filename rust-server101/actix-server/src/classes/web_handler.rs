use actix::prelude::*;
use ring::{digest, pbkdf2};
use std::num::NonZeroU32;
use data_encoding::HEXLOWER;
use mongodb::{bson::doc, Client,};

use super::shared_struct::LoggedInformation;

pub struct WebHandler {
    
}

impl WebHandler {
    // given a valid cookie, html with information from state server
    // should be provided
    pub fn get_info(_res_id: String) {

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
                    eprintln!("No sensors found for res_id: {}", username);
                    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "no user found"))
                }
                Err(err) => {
                    eprintln!("Error querying sensors: {:?}", err);
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


