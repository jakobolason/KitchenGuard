use mongodb::Client;
use super::{
    job_scheduler::JobsScheduler,
    state_handler::StateHandler,
    web_handler::WebHandler,
};

use std::pin::Pin;
use ring::{digest, pbkdf2};
use std::num::NonZeroU32;
use data_encoding::HEXLOWER;
use serde::{Deserialize, Serialize};
use actix::Message;
use actix_web::HttpResponse;
use crate::classes::state_handler::Event;

pub struct AppState {
    pub state_handler: actix::Addr<StateHandler>,
    pub job_scheduler: actix::Addr<JobsScheduler>,
    pub web_handler: actix::Addr<WebHandler>,
    pub db_client: Client,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Message)]
#[rtype(result = "Option<Vec<Event>>")]
pub struct ResUidFetcher {
    pub res_uid: String,
}

pub fn hash_password(password: &str, salt: &[u8]) -> String {
    // Configure PBKDF2 parameters
    let n_iter = NonZeroU32::new(100_000).unwrap();
    let alg = pbkdf2::PBKDF2_HMAC_SHA256;
    
    // Output buffer for the hash
    let mut hash = [0u8; digest::SHA256_OUTPUT_LEN];
    
    pbkdf2::derive(
        alg,
        n_iter,
        salt,
        password.as_bytes(),
        &mut hash,
    );
    
    // Convert to hex string
    HEXLOWER.encode(&hash)
}

// For saving login informatino in db
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct LoggedInformation {
    pub username: String,
    pub password: String,
    pub salt: Vec<u8>,
    pub res_ids: Vec<String>,
}

// What the user queries the server with
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Message)]
#[rtype(result = "Option<String>")]
pub struct LoginInformation {
    pub username: String,
    pub password: String,
}

#[derive(Message)]
#[rtype(result = "Option<Vec<String>>")]
pub struct ValidateSession {
    pub cookie: String
}