use actix::prelude::*;
use chrono::DateTime;
use actix_web::web::Data;
use serde::{Deserialize, Serialize};
use mongodb::{bson::doc, Client};

use super::job_scheduler::{self, JobsScheduler};

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

// ============ Messages across Actors =============

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

impl Message for Event {
    type Result = ();
}

#[derive(Debug)]
pub struct JobCompleted {
    res_id: String,
}

impl Message for JobCompleted {
    type Result = ();
}

#[derive(Clone)]
pub struct StateHandler {
    db_client: Client,
    job_scheduler: Option<Addr<JobsScheduler>>, 
}

impl Actor for StateHandler {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Statehandler actor started!");
    }
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

    fn determine_new_state(current_state: &States, list_of_sensors: &SensorLookup, data: &Event) -> States{
        // IF were in any of these states, then we only check if it's kitchen PIR detecting motion
        if *current_state == States::CriticallyAlarmed || *current_state == States::Alarmed
            || *current_state == States::Unattended 
        {
            // if event is elderly moving into kitchen, then turn off alarm
            if data.device_model == list_of_sensors.kitchen_pir && data.mode == "true" { // occupancy: true
                if *current_state == States::Unattended || *current_state == States::Alarmed {
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
                current_state.clone()
            }
        // In attended, we check both kitchen PIR status and power plug status
        } else if *current_state == States::Attended {
            // if user is entering kitche, then cancel jobscheduler timer 
            if data.device_model == list_of_sensors.kitchen_pir && data.mode == "false" { // occupancy: false
                // TODO: then start 20min's timer in jobscheduler
                States::Unattended
            } else if data.device_model == list_of_sensors.power_plug && data.mode == "Off" {
                States::Standby
            } else {
                // else do nothing, could be other room PIR saying somethin
                current_state.clone()
            }
        // If power plug gets turned on
        } else if *current_state == States::Standby {
            if data.device_model == list_of_sensors.power_plug && data.mode == "On" {
                States::Attended
            } else {
                current_state.clone()
            }
            // Should not be possible, but just in case
        } else {
            // default to current_state
            current_state.clone()
        }
    }

    async fn get_resident_data(res_id: String, db_client: Client) -> Result<(States, SensorLookup), mongodb::error::Error> {
        // Fetch the current state
        let state_collection = db_client.database("States").collection::<StateLog>(&res_id);
        let current_state = match state_collection.find_one(doc! {"res_id": &res_id}).await {
            Ok(Some(document)) => document.state,
            Ok(None) => {
                eprintln!("No state found for res_id: {}", res_id);
                States::Standby
            }
            Err(err) => {
                eprintln!("Error querying state: {:?}", err);
                return Err(err);
            }
        };

        // Fetch the list of sensors
        let sensor_collection = db_client.database("SensorLookup").collection::<SensorLookup>(&res_id);
        let sensors = match sensor_collection.find_one(doc! {"res_id": &res_id}).await {
            Ok(Some(document)) => document,
            Ok(None) => {
                eprintln!("No sensors found for res_id: {}", res_id);
                SensorLookup {
                    kitchen_pir: String::new(),
                    power_plug: String::new(),
                    other_pir: Vec::new(),
                    led: Vec::new(),
                    speakers: Vec::new(),
                }
            }
            Err(err) => {
                eprintln!("Error querying sensors: {:?}", err);
                return Err(err);
            }
        };

        Ok((current_state, sensors))
    }
}

impl Handler<Event> for StateHandler {
    type Result = ();

    fn handle(&mut self, data: Event, _ctx: &mut Self::Context) {
        let db_client = self.db_client.clone();
        let job_scheduler = self.job_scheduler.clone();
        let res_id = data.res_id.to_string();

        // Actor handling doesn't implement async functionality, so do some move magix
        Box::pin(async move {
            // Log the event data
            let collection = db_client.database("Residents").collection::<Event>(&res_id);
            if let Err(err) = collection.insert_one(data.clone()).await {
                eprintln!("Failed to log event: {:?}", err);
                return Err(format!("Failed to log event: {:?}", err));
            }
            let (current_state, sensors) = match StateHandler::get_resident_data(res_id.clone(), db_client.clone()).await {
                Ok(vals) => (vals.0, vals.1),
                Err(_err) => panic!("couldn't get resident data"),
            };

            // Determine the new state, !! TODO: should also return if any instruction to job scheduler exists
            let new_state = StateHandler::determine_new_state(&current_state, &sensors, &data);

            // Save the new state
            let state_log = StateLog {
                res_id: res_id.clone(),
                timestamp: chrono::Utc::now(),
                state: new_state.clone(),
                context: format!("{:?}", data),
            };
            let state_collection = db_client.database("States").collection::<StateLog>(&res_id);
            if let Err(err) = state_collection.insert_one(state_log).await {
                eprintln!("Failed to save new state: {:?}", err);
                return Err(format!("Failed to save new state: {:?}", err));
            };
            Ok(())
        }
        .into_actor(self)) // Convert the future into an actor future
        .map(|res, _, _| {
            if let Err(err) = res {
                eprintln!("Error in actor future: {:?}", err);
            }
        });
        ()
    }
}

impl Handler<JobCompleted> for StateHandler {
    type Result = ();

    fn handle(&mut self, msg: JobCompleted, _ctx: &mut Self::Context) {
        println!("A job was completed! res_id: {:?}", msg);
    }
}

// enum