use mongodb::{bson::Binary, Client};
use chrono::DateTime;

use super::{
    state_handler::StateHandler,
    web_handler::WebHandler,
};

use ring::{digest, pbkdf2};
use std::num::NonZeroU32;
use data_encoding::HEXLOWER;
use serde::{Deserialize, Serialize};
use actix::Message;
use std::time::Instant;

pub static MONGODB_URI: &str = "mongodb://localhost:27017";
/// This holds the collections holding information for residents
pub static RESIDENT_DATA: &str = "resident_data";
pub static RESIDENT_LOGS: &str = "resident_logs";
pub static DEVICE_HEALTH: &str = "device_health";
pub static STATES: &str = "states";
pub static SENSOR_LOOKUP: &str = "sensor_lookup";
pub static IP_ADDRESSES: &str = "ip_addresses";

/// This holds information on users/relatives, and their login information
pub static USERS: &str = "users";
pub static INFO: &str = "info";

/// The endpoint configured on the Pi
pub static PI_LISTENER: &str = "state_listener";
pub static SMS_SERVICE: &str = "https://api.twilio.com/2010-04-01/Accounts/";

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

pub struct AppState {
    pub state_handler: actix::Addr<StateHandler>,
    pub web_handler: actix::Addr<WebHandler>,
    pub db_client: Client,
}

// HEUCOD event standard, needs implementing.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Event {
    pub time_stamp: String,
    pub mode: String,
    pub event_data: String,
    pub event_type_enum: String, // Or we could define an enum here
    pub res_id: String, // changed from patient_id to res_id
    pub device_model: String,
    pub device_vendor: String,
    pub gateway_id: u32,
    pub id: String,
}
impl Message for Event {type Result = Result<States, std::io::ErrorKind>; }


#[derive(PartialEq, Eq, Deserialize, Serialize, Clone, Debug, Ord, PartialOrd)]
pub enum States {
    Initialization = 0,
    Standby = 1,
    Attended = 2,
    Unattended = 3,
    Alarmed = 4,
    CriticallyAlarmed = 5,
    Faulty = 6,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct HealthCheck {
    pub res_id: String,
    pub data: Vec<(String, String)>
}
impl Message for HealthCheck { type Result = (); }

// For saving login informatino in db
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct UsersLoggedInformation {
    pub username: String,
    pub password: String,
    pub salt: Binary,
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

// holds lists for a residents devices. Note that requirements state we need 5 PIR sensors
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct SensorLookup {
    pub res_id: String,
    pub kitchen_pir: String,
    pub power_plug: String,
    pub other_pir: Vec<String>, // a good idea would be to index the rooms pir, speaker and LED with same index
    pub led: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Message)]
#[rtype(result = "()")]
pub struct InitState {
    pub info: SensorLookup,
    pub ip_addr: String,
}

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct ScheduledTask {
  pub res_id: String,
  pub execute_at: Instant,
}

// #[derive(Debug, Message, Clone)]
// #[rtype(result = ())]
// pub struct NotifyRelatives {}

#[derive(Message, Deserialize)]
#[rtype(result = "Option<String>")]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub phone_number: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct IpCollection {
    pub res_ip: String,
    pub res_id: String,
}

#[derive(Message, Deserialize)]
#[rtype(result = "()")]
pub struct AddRelative {
    pub res_id: String,
    pub username: String,
}

// For when an alarm is sounded
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct StateLog {
    pub res_id: String,
    pub timestamp: DateTime<chrono::Utc>,
    pub state: States,
    pub current_room_pir: String,
    pub context: String,            // Store full system state snapshot here
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ResIdFetcher {
    pub res_id: String,
}
impl Message for ResIdFetcher {type Result = Option<Vec<StateLog>>;}

#[derive(Deserialize, Serialize)]
pub struct GetStoveData {
    pub res_id: String,
}
impl Message for GetStoveData {type Result = Option<Vec<Event>>;}

#[derive(Deserialize, Serialize)]
pub struct GetHealthData {
    pub res_id: String,
}
impl Message for GetHealthData {type Result = Option<HealthCheck>;}

#[derive(Deserialize, Serialize)]
pub struct TurnOffalarm {
    pub res_id: String
}
impl Message for TurnOffalarm {type Result = (); }