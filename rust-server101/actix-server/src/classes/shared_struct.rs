use mongodb::Client;
use super::{
    job_scheduler::JobsScheduler,
    state_handler::StateHandler,
    web_handler::WebHandler,
};

use ring::{digest, pbkdf2};
use std::num::NonZeroU32;
use data_encoding::HEXLOWER;
use serde::{Deserialize, Serialize};
use actix::Message;
use std::time::Instant;
/// This holds the collections holding information for residents
pub static ResidentData: &str = "ResidentData";
pub static States: &str = "States";
pub static SensorLookup: &str = "SensorLookup";
pub static ip_addresses: &str = "ip_addresses";

/// This holds information on users/relatives, and their login information
pub static users: &str = "users";
pub static info: &str = "info";

/// The endpoint configured on the Pi
pub static pi_listener: &str = "state_listener";
pub static sms_service: &str = "https://api.twilio.com/2010-04-01/Accounts/";


pub struct AppState {
    pub state_handler: actix::Addr<StateHandler>,
    pub job_scheduler: actix::Addr<JobsScheduler>,
    pub web_handler: actix::Addr<WebHandler>,
    pub db_client: Client,
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
    pub phone_number: String,
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

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct ScheduledTask {
  pub res_id: String,
  pub execute_at: Instant,
}

#[derive(Message, Deserialize)]
#[rtype(result = "Option<String>")]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub phone_number: String,
}
