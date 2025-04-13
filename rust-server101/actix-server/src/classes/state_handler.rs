use actix::prelude::*;
use actix_web::rt::task;
use chrono::DateTime;
// use actix_web::{cookie::time::Duration, rt::task, web::Data};
use serde::{Deserialize, Serialize};
use mongodb::{bson::doc, Client};
use std::time::{Duration, Instant};

use super::job_scheduler::{JobsScheduler, ScheduledTask, CancelTask};

#[derive(Eq, PartialEq)]
// An extension to scheduled task, to incorporate the type returned  (if it's a new task or a cancellation)
enum TypeOfTask {
    Cancellation,
    NewTask,
    None,
}

struct TaskValue {
    type_of_task: TypeOfTask,
    scheduled_task: Option<ScheduledTask>, // only used if type is NewTask
    res_id: String,
}

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
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Message)]
#[rtype(result = "()")]
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

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct JobCompleted {
    pub res_id: String,
}


#[derive(Clone)]
pub struct StateHandler {
    pub db_client: Client,
    pub job_scheduler: Option<Addr<JobsScheduler>>, 
}

impl Actor for StateHandler {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Statehandler actor started!");
    }
}

// ===== FOR SETTING OF SCHEDULER TO HANDLER =====
#[derive(Debug)]
pub struct SetJobScheduler {
    pub scheduler: Option<Addr<JobsScheduler>>,
}
impl Message for SetJobScheduler {
    type Result = ();
}
impl Handler<SetJobScheduler> for StateHandler {
    type Result = ();

    fn handle(&mut self, scheduler: SetJobScheduler, _ctx: &mut Self::Context) {
        self.job_scheduler = scheduler.scheduler;
        
        ()
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

    fn determine_new_state(current_state: &States, list_of_sensors: &SensorLookup, data: &Event) -> (States, TaskValue) {
        let mut scheduled_task = TaskValue {
            type_of_task: TypeOfTask::None,
            scheduled_task: None,
            res_id: String::new(),
        };
        // IF were in any of these states, then we only check if it's kitchen PIR detecting motion
        let new_state = if *current_state == States::CriticallyAlarmed || *current_state == States::Alarmed
                                            || *current_state == States::Unattended 
        {
            // if event is elderly moving into kitchen, then turn off alarm
            if data.device_model == list_of_sensors.kitchen_pir && data.mode == "true" { // occupancy: true
                if *current_state == States::Unattended || *current_state == States::Alarmed {
                    scheduled_task = TaskValue {
                        type_of_task: TypeOfTask::Cancellation,
                        scheduled_task: None,
                        res_id: data.res_id.to_string().clone(),
                    }
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
            if data.device_model == list_of_sensors.kitchen_pir && data.mode == "false" { // occupancy: false
                // TODO: then start 20min's timer in jobscheduler
                scheduled_task = TaskValue {
                    type_of_task: TypeOfTask::NewTask,
                    scheduled_task: Some(ScheduledTask {
                        res_id: data.res_id.to_string().clone(),
                        execute_at: Instant::now() + Duration::from_secs(20 * 60),
                    }),
                    res_id: data.res_id.to_string().clone(),
                };
                States::Unattended
                // If elderly turns off the stove
            } else if data.device_model == list_of_sensors.power_plug && data.mode == "Off" { // power: Off
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
        };
        return (new_state, scheduled_task)
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
        let job_scheduler = match self.job_scheduler.clone() {
            Some(jobber) => jobber,
            _=> { 
                eprint!("There was no job scheduler address available!");
                return
            }
        };
        let res_id = data.res_id.to_string();

        // Actor handling doesn't implement async functionality, so do some move magix
        let fut = async move {
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
                state: new_state.0.clone(),
                context: format!("{:?}", data),
            };
            // if any job scheduling task -- either new task(20 minutes) or a cancellation
            if new_state.1.type_of_task != TypeOfTask::None {
                let task_el = new_state.1;
                if task_el.type_of_task == TypeOfTask::NewTask {
                   match task_el.scheduled_task {
                        Some(task) => job_scheduler.do_send(task),
                        _=> eprint!("Tried to queue task, but was missing it")
                    }
                } else if task_el.type_of_task == TypeOfTask::Cancellation {
                    let res_id = task_el.res_id;
                    job_scheduler.do_send(CancelTask {
                        res_id,
                    });
                } else {
                    // do nothing i guess
                }
            }
            let state_collection = db_client.database("States").collection::<StateLog>(&res_id);
            if let Err(err) = state_collection.insert_one(state_log).await {
                eprintln!("Failed to save new state: {:?}", err);
                return Err(format!("Failed to save new state: {:?}", err));
            };
            Ok(())
        }
        .into_actor(self) // Convert the future into an actor future
        .map(|res, _, _| {
            if let Err(err) = res {
                eprintln!("Error in actor future: {:?}", err);
            }
        });

        _ctx.spawn(fut);
        ()
    }
}

impl Handler<JobCompleted> for StateHandler {
    type Result = ();

    fn handle(&mut self, msg: JobCompleted, _ctx: &mut Self::Context) {
        println!("A job was completed! res_id: {:?}", msg.res_id);
    }
}

// enum