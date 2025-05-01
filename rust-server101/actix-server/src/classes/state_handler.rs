use actix::{Actor, Addr, Context, Handler, Message, ResponseFuture};
use chrono::DateTime;
// use actix_web::{cookie::time::Duration, rt::task, web::Data};
use serde::{Deserialize, Serialize};
use mongodb::{bson::{oid::ObjectId, doc}, Client, error};
use core::panic;
use std::time::{Duration, Instant};
use std::pin::Pin;

use super::{
    job_scheduler::{JobsScheduler, CancelTask}, 
    shared_struct::{LoggedInformation, LoginInformation, ScheduledTask, hash_password},
    pi_communicator::PiCommunicator,
};

#[derive(Eq, PartialEq, Debug)]
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

impl TaskValue {
    fn new() -> TaskValue {
        TaskValue {
            type_of_task: TypeOfTask::None,
            scheduled_task: None,
            res_id: "-1".to_string(),
        }
    }
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
    pub _id: ObjectId,
    pub res_id: String,
    pub kitchen_pir: String,
    pub power_plug: String,
    pub other_pir: Vec<String>, // a good idea would be to index the rooms pir, speaker and LED with same index
    pub led: Vec<String>,
    pub speakers: Vec<String>, 
}

// For when an alarm is sounded
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct StateLog {
    pub _id: ObjectId,
    pub res_id: String,
    pub timestamp: DateTime<chrono::Utc>,
    pub state: States,
    pub context: String,            // Store full system state snapshot here
}


// ============ Messages across Actors =============

// HEUCOD event standard, needs implementing.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Message)]
#[rtype(result = "Result<States, std::io::ErrorKind>")]
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

// ============= Setup of StateHandler =============
#[derive(Clone)]
pub struct StateHandler {
    pub db_client: Client, 
    pub job_scheduler: Option<Addr<JobsScheduler>>, // option, because we don't have address of scheduler at initialisation (look in main.rs)
    pub is_test: bool,
}
// Makes StateHandler like a CPU from vdm-rt, but no bus is needed in Rust. Instead messages are used. Look for `impl Handler for StateHandler{` to see how that is done
impl Actor for StateHandler {
    type Context = Context<Self>;
    // deploys the statehandler on a thread, listenin for messages
    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Statehandler actor started!");
    }
}

// ===== FOR SETTING OF SCHEDULER TO HANDLER =====
#[derive(Debug, Message)]
#[rtype(result = "()")] //
pub struct SetJobScheduler {
    pub scheduler: Option<Addr<JobsScheduler>>,
}

impl Handler<SetJobScheduler> for StateHandler {
    type Result = ();

    fn handle(&mut self, scheduler: SetJobScheduler, _ctx: &mut Self::Context) {
        self.job_scheduler = scheduler.scheduler;
        ()
    }
}

impl StateHandler {

//     // somehow notify them
//     fn notify_relatives(res_id: String) {

//     }


    fn alarm_duration_from_state(new_state: States, is_test: bool) -> Instant {
        if is_test {println!("we are in tests");}
        else { println!("not in tests...");}
        match new_state {
            States::Unattended => Instant::now() + if is_test { Duration::from_secs(4) } else { Duration::from_secs(20 * 60) },
            States::Alarmed => Instant::now() + if is_test { Duration::from_secs(3) } else { Duration::from_secs(20 * 3) },
            _ => panic!("You should not give state '{:?}' to this function!", new_state),
        }
    }

    /// This function is our main business logic, determining what we should do given a state and an event happening.
    /// Returns: the new state for the resident, and maybe a task to be scheduled, cancelled or to do nothing.
    fn determine_new_state(current_state: &States, list_of_sensors: &SensorLookup, data: &Event, is_test: bool) -> (States, TaskValue) {
        println!("Current state: {:?}", *current_state);
        println!("current mode: {:?} and sensor: {:?}", data.mode, data.device_model);
        let mut scheduled_task = TaskValue::new();
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
            } else if data.device_model == "JobScheduler" {
                println!("device found to be from jobscheduler");
                // An event from jobscheduler means that a timer was done, so give the appropriate task duration
                let next_state = match current_state {
                    States::Alarmed => States::CriticallyAlarmed,
                    States::Unattended => States::Alarmed,
                    _ => {
                        eprintln!("Unexpected state encountered: {:?}", current_state);
                        panic!("Invalid state transition detected");
                    }
                };
                scheduled_task = TaskValue {
                    type_of_task: TypeOfTask::NewTask,
                    scheduled_task: Some(ScheduledTask {
                        res_id: data.res_id.to_string().clone(),
                        execute_at: StateHandler::alarm_duration_from_state(next_state.clone(), is_test),
                    }),
                    res_id: data.res_id.to_string().clone(),
                };
                next_state

            } else {
                // if it's not the user moving into kitchen, don't do anything
                current_state.clone()
            }
        // In attended, we check both kitchen PIR status and power plug status
        } else if *current_state == States::Attended {
            if data.device_model == list_of_sensors.kitchen_pir && data.mode == "false" { // occupancy: false
                println!("resident is out of kitchen!");
                // then start 20min's timer in jobscheduler
                scheduled_task = TaskValue {
                    type_of_task: TypeOfTask::NewTask,
                    scheduled_task: Some(ScheduledTask {
                        res_id: data.res_id.to_string().clone(),
                        execute_at: StateHandler::alarm_duration_from_state(States::Unattended, is_test),
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

    async fn get_resident_data(res_id: String, db_client: Client) -> error::Result<(States, SensorLookup)> {
        // Fetch the current state
        let state_collection = db_client.database("ResidentData").collection::<StateLog>("States");
        let current_state = match state_collection
            .find_one(doc! {"res_id": &res_id})
            .sort(doc!{"_id": -1}) //finds the latest (datewise) entry matching res_id
            .await {
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
        let sensor_collection = db_client.database("ResidentData").collection::<SensorLookup>("SensorLookup");
        let sensors = match sensor_collection
            .find_one(doc! {"res_id": &res_id})
            .sort(doc!{"_id": -1})
            .await {
            Ok(Some(document)) => document,
            Ok(None) => {
                eprintln!("No sensors found for res_id: {}", res_id);
                SensorLookup {
                    _id: ObjectId::new(),
                    res_id: res_id.clone(),
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

    
    pub async fn create_user(username: &str, password: &str, db_client: Client) -> Option<mongodb::results::InsertOneResult> {
        let user_salt = username.as_bytes();
        let hashed_password =  hash_password(password, user_salt);
        let usercollection = db_client.database("users").collection::<LoggedInformation>("info");
        println!("creating user");
        usercollection.insert_one(LoggedInformation {
            username: username.to_string(),
            password: hashed_password,
            salt: user_salt.to_vec(),
            res_ids: Vec::new(), // idk how to handle this best
        }).await.map_err(|e| {
            eprintln!("Failed to insert user: {:?}", e);
        }).ok()
    }
    
}

impl Handler<LoginInformation> for StateHandler {
    type Result = Option<String>;

    fn handle(&mut self, data: LoginInformation, _ctx: &mut Self::Context ) -> Self::Result {
        println!("creating user");
        let db_client = self.db_client.clone();
        actix::spawn(async move {
            StateHandler::create_user(&data.username, &data.password, db_client).await;
        });
        Some("OK".to_string())
    }
}

/// Handles what to do, when a sensor event comes from the Pi
/// It does way too much in one function, but the actor framework 
/// Returns a future that should be awaited
impl Handler<Event> for StateHandler {
    type Result = ResponseFuture<Result<States, std::io::ErrorKind>>;

    fn handle(&mut self, data: Event, _ctx: &mut Self::Context) -> Self::Result {
        println!("Caught an event for res_id: {:?}", data.res_id);
        let db_client = self.db_client.clone();
        let job_scheduler = self.job_scheduler.clone().unwrap();
        let res_id = data.res_id.to_string();
        let is_test = self.is_test.clone();

        // Returns a future, that should be awaited
        Box::pin(async move {
            // Log the event data
            let collection = db_client.database("ResidentData").collection::<Event>("ResidentLogs");
            if let Err(err) = collection.insert_one(data.clone()).await {
                println!("In error");
                eprintln!("Failed to log event: {:?}", err);
                return Err(std::io::ErrorKind::InvalidData);
            }
            let (current_state, sensors) = match StateHandler::get_resident_data(res_id.clone(), db_client.clone()).await {
                Ok(vals) => (vals.0, vals.1),
                Err(_err) => return Err(std::io::ErrorKind::InvalidInput),
            };
            // Determine the new state, !! TODO: should also return if any instruction to job scheduler exists
            let (state_info, task_type) = StateHandler::determine_new_state(&current_state, &sensors, &data, is_test.clone());
            println!("new state found to be: {:?}", state_info);
            // Save the new state
            let state_log = StateLog {
                _id: ObjectId::new(),
                res_id: res_id.clone(),
                timestamp: chrono::Utc::now(),
                state: state_info.clone(),
                context: format!("{:?}", data),
            };
            // if any job scheduling task -- either new task(20 minutes) or a cancellation
            if task_type.type_of_task != TypeOfTask::None {
                let task_el = task_type;
                if task_el.type_of_task == TypeOfTask::NewTask {
                    println!("Sending task to scheduler!");
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
            let state_collection = db_client.database("ResidentData").collection::<StateLog>("States");
            if let Err(err) = state_collection.insert_one(state_log).await {
                eprintln!("Failed to save new state: {:?}", err);
                return Err(std::io::ErrorKind::InvalidInput);
            };
            // This isn't optimal, and would've been nice to be able to put in the requester's flow instead of here
            // send new state_info to pi communicator
            PiCommunicator::send_new_state(res_id, state_info.clone(), db_client.clone()).await;
            Ok(state_info)
        })
    }
}

// ====== TESTING ======
#[cfg(test)]
mod tests {
	use super::*;

    #[test]
    fn test_determine_new_state_critically_alarmed_to_standby() {
        let current_state = States::CriticallyAlarmed;
        let list_of_sensors = SensorLookup {
            _id: ObjectId::new(),
            res_id: "1".to_string(),
            kitchen_pir: "kitchen_pir_1".to_string(),
            power_plug: "power_plug_1".to_string(),
            other_pir: vec![],
            led: vec![],
            speakers: vec![],
        };
        let data = Event {
            time_stamp: "2023-01-01T00:00:00Z".to_string(),
            mode: "true".to_string(),
            event_data: "".to_string(),
            event_type_enum: "".to_string(),
            res_id: "".to_string(),
            device_model: "kitchen_pir_1".to_string(),
            device_vendor: "".to_string(),
            gateway_id: 1,
            id: "".to_string(),
        };

        let (new_state, task_value) = StateHandler::determine_new_state(&current_state, &list_of_sensors, &data, true);

        assert_eq!(new_state, States::Standby);
        assert_eq!(task_value.type_of_task, TypeOfTask::None);
    }

    #[test]
    fn test_determine_new_state_attended_to_unattended() {
        let current_state = States::Attended;
        let list_of_sensors = SensorLookup {
            _id: ObjectId::new(),
            res_id: "1".to_string(),
            kitchen_pir: "kitchen_pir_1".to_string(),
            power_plug: "power_plug_1".to_string(),
            other_pir: vec![],
            led: vec![],
            speakers: vec![],
        };
        let data = Event {
            time_stamp: "2023-01-01T00:00:00Z".to_string(),
            mode: "false".to_string(),
            event_data: "".to_string(),
            event_type_enum: "".to_string(),
            res_id: "".to_string(),
            device_model: "kitchen_pir_1".to_string(),
            device_vendor: "".to_string(),
            gateway_id: 1,
            id: "".to_string(),
        };

        let (new_state, task_value) = StateHandler::determine_new_state(&current_state, &list_of_sensors, &data, true);

        assert_eq!(new_state, States::Unattended);
        assert_eq!(task_value.type_of_task, TypeOfTask::NewTask);
        assert!(task_value.scheduled_task.is_some());
    }

    #[test]
    fn test_determine_new_state_standby_to_attended() {
        let current_state = States::Standby;
        let list_of_sensors = SensorLookup {
            _id: ObjectId::new(),
            res_id: "1".to_string(),
            kitchen_pir: "kitchen_pir_1".to_string(),
            power_plug: "power_plug_1".to_string(),
            other_pir: vec![],
            led: vec![],
            speakers: vec![],
        };
        let data = Event {
            time_stamp: "2023-01-01T00:00:00Z".to_string(),
            mode: "On".to_string(),
            event_data: "".to_string(),
            event_type_enum: "".to_string(),
            res_id: "".to_string(),
            device_model: "power_plug_1".to_string(),
            device_vendor: "".to_string(),
            gateway_id: 1,
            id: "".to_string(),
        };

        let (new_state, task_value) = StateHandler::determine_new_state(&current_state, &list_of_sensors, &data, true);

        assert_eq!(new_state, States::Attended);
        assert_eq!(task_value.type_of_task, TypeOfTask::None);
    }

    #[test]
    fn test_determine_new_state_no_change() {
        let current_state = States::Attended;
        let list_of_sensors = SensorLookup {
            _id: ObjectId::new(),
            res_id: "1".to_string(),
            kitchen_pir: "kitchen_pir_1".to_string(),
            power_plug: "power_plug_1".to_string(),
            other_pir: vec![],
            led: vec![],
            speakers: vec![],
        };
        let data = Event {
            time_stamp: "2023-01-01T00:00:00Z".to_string(),
            mode: "true".to_string(),
            event_data: "".to_string(),
            event_type_enum: "".to_string(),
            res_id: "".to_string(),
            device_model: "other_pir_1".to_string(),
            device_vendor: "".to_string(),
            gateway_id: 1,
            id: "".to_string(),
        };

        let (new_state, task_value) = StateHandler::determine_new_state(&current_state, &list_of_sensors, &data, true);

        assert_eq!(new_state, States::Attended);
        assert_eq!(task_value.type_of_task, TypeOfTask::None);
    }	

}