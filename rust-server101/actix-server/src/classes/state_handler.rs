use std::collections::HashMap;
use chrono::DateTime;
use actix_web::web::Data;
use serde::{Deserialize, Serialize};
use mongodb::{bson, bson::doc, Client};

#[derive(PartialEq, Eq, Deserialize, Serialize, Clone, Debug)]
pub enum States {
    Standby,
    Attended,
    Unattended,
    Alarmed,
    CriticallyAlarmed
}

// holds lists for a residents devices. Note that requirements state we need 5 PIR sensors
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct SensorLookup {
    kitchen_pir: String,
    power_plug: String,
    other_pir: Vec<String>, // a good idea would be to index the rooms pir, speaker and LED with same index
    led: Vec<String>,
    speakers: Vec<String>, 
}

pub enum SensorType {
    PIR,
    PP, // PowerPlug
    LED,
    SP, // speaker
}

// For when an alarm is sounded
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
struct StateLog {
    res_id: String,
    timestamp: DateTime<chrono::Utc>,
    state: States,
    context: String,            // Store full system state snapshot here
}
// HEUCOD event standard, needs implementing.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Event {
    pub time_stamp: String,
    pub mode: String,
    pub event_data: String,
    pub event_type_enum: String, // Or we could define an enum here
    pub res_id: u32, // changed from patient_id to res_id
    pub device_model: String,
    pub device_vendor: String,
    pub gateway_id: u32,
    pub id: String,
}

#[derive(Clone)]
pub struct StateHandler {
}


impl StateHandler {
// Private functions
    // sends topic to turn off stove
    fn turn_off_stove() {
        // Pre: state in [Unattended, Alarmed, CriticallyAlarmed]
        // Post: stove is off

    }
    // sends topic to LED's and buzzer sound
    fn begin_alarm() {
        // Pre: state in [Unattended]
        // Post: timer started for CriticallyAlarmed & LED turned on

    }
    // sends topic to powerplug to turn off, and
    fn critical_alarm() {
        // Pre: state in [Alarmed]
        // Post: timer started to notify relatives & stove turned off

    }
    // stop alarm after elderly has returned to kitchen
    fn stop_alarm() {
        // Pre: state in [Alarmed, CriticallyAlarmed] & user is in kitchen
        // Post: state should be changed, LED turned off

    }
    // get database information on PIR sensor data
    fn check_users_room() {
        // Pre: none
        // Post: 

    }
    // somehow notify them
    fn notify_relatives(res_id: String) {

    }
    // make server listen to a topic
    fn sub(topic: String) {

    }
    // publish a topic(used in alarm e.g.)
    fn publish(topic: String) {
        
    }
    // This handles db connection and querying and response handling
    fn db_query() { // don't know if hashmap is the best here

    }
    // check users given credentials
    // NOTE: Should maybe return both access token and a list of strings
//           for elder_uids
    fn check_credentials(user: String, pwd: String) {

    }
    // start a thread, that makes callbacks when 20 minutes ha spassed
    fn start_clock() {

    }

// Public functions
pub fn new() -> Self {
        // Initialization logic here
        StateHandler {
            // Initialize fields if any
        }
    }
    pub async fn event(data: &Event, res_id: &str, client: Data<Client>) {
        // check res_id is a registered resident in db

        // log the event data
        let collection = client.database("Residents").collection::<Event>(&res_id);
        let result = collection.insert_one(data).await;
        match result {
            Ok(_) => println!("Logged the event correctly"), 
            Err(err) => panic!("Log of event failed with error:\n {:?}", err),
        }
        // Get state from db, maybe check the latest entry instead of only 1 entry
        let collection = client.database("States").collection::<StateLog>(&res_id);
        let current_state: States = match collection.find_one(doc! {"res_id": res_id}).await {
            Ok(Some(document)) => {
                // Deserialize the document into a StateLog and extract the state
                println!("Current state captured: {:?}", document.state);
                document.state
            }
            Ok(None) => {
            println!("No document found with the given query.");
            return; // Handle no document case appropriately
            }
            Err(e) => {
            eprintln!("Error occurred while querying: {}", e);
            return; // Handle query error appropriately
            }
        };

        // Get the list of sensors for resident
        let collection = client.database("SensorLookup").collection::<SensorLookup>(&res_id);
        let list_of_sensors: SensorLookup = match collection.find_one(doc! {"res_id": res_id}).await {
            Ok(Some(document)) => {
                // Document found, print it
                println!("Sensor lookup captured: {:?}", document);
                document
            }
            Ok(None) => {
                // No document found
                println!("No document found with the given query.");
                panic!()
            }
            Err(e) => {
                // Handle any errors
                eprintln!("Error occurred while querying: {}", e);
                panic!()
            }
        };
        // this if/else statement returns the new state
        let new_state: States = 
            // IF were in any of these states, then we only check if it's kitchen PIR detecting motion
            if current_state == States::CriticallyAlarmed || current_state == States::Alarmed
                                                         || current_state == States::Unattended 
            {
                // if event is elderly moving into kitchen, then turn off alarm
                if data.device_model == list_of_sensors.kitchen_pir && data.mode == "true" { // occupancy: true
                    if current_state == States::Unattended || current_state == States::Alarmed {
                        // TODO: cancel jobscheduler timer given the res_id
                    }
                    // then go into Standby/Stove-attended according to state
                    match current_state {
                        States::CriticallyAlarmed => States::Standby,
                        States::Alarmed => States::Attended,
                        States::Unattended => States::Attended,
                        _ => {
                            eprintln!("Unexpected state encountered: {:?}", current_state);
                            panic!("Invalid state transition detected");
                        }
                    }
                } else {
                    // if it's not the user moving into kitchen, don't do anything
                    current_state
                }
            // In attended, we check both kitchen PIR status and power plug status
            } else if current_state == States::Attended {
                // if user is entering kitche, then cancel jobscheduler timer 
                if data.device_model == list_of_sensors.kitchen_pir && data.mode == "false" { // occupancy: false
                    // TODO: then start 20min's timer in jobscheduler
                    States::Unattended
                } else if data.device_model == list_of_sensors.power_plug && data.mode == "Off" {
                    States::Standby
                } else {
                    // else do nothing, could be other room PIR saying somethin
                    current_state
                }
            // If power plug gets turned on
            } else if current_state == States::Standby {
                if data.device_model == list_of_sensors.power_plug && data.mode == "On" {
                    States::Attended
                } else {
                    current_state
                }
            // Should not be possible, but just in case
            } else {
                // default to current_state
                current_state
            };

        // now insert the new state
        // TODO: Determine whether we should update db, if it's the same state
        let state_collection = client.database("States").collection::<StateLog>(&res_id);
        let state_log = StateLog {
            res_id: res_id.to_string(),
            timestamp: chrono::Utc::now(),
            state: new_state.clone(),
            context: format!("{:?}", data),
        };
        let result = state_collection.insert_one(state_log).await;
        match result {
            Ok(_) => println!("Changed the state to {:?}", new_state), 
            Err(err) => panic!("Failed to save new state, given error: {:?}", err),
        }
    }




}

// enum