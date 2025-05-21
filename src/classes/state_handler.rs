use actix::{Actor, Addr, Context, Handler, Message, ResponseFuture};
use mongodb::{bson::{self, doc}, Client};
use core::panic;
use std::time::{Duration, Instant};
use std::env;
use dotenv::dotenv;
use futures_util::StreamExt;

use super::{
    job_scheduler::{CancelTask, JobsScheduler}, pi_communicator::PiCommunicator, 
    shared_struct::{self, AddRelative, HealthCheck, ScheduledTask, SensorLookup, StateLog, UsersLoggedInformation},
};

#[derive(Eq, PartialEq, Debug)]
// An extension to scheduled task, to incorporate the type returned  (if it's a new task or a cancellation)
enum TypeOfTask {
    Cancellation,
    NewTask,
}

struct TaskValue {
    type_of_task: TypeOfTask,
    scheduled_task: Option<shared_struct::ScheduledTask>, // only used if type is NewTask
    res_id: String,
}

// ============= Setup of StateHandler =============
#[derive(Clone)]
pub struct StateHandler {
    db_client: Client, 
    job_scheduler: Option<Addr<JobsScheduler>>, // option, because we don't have address of scheduler at initialisation (look in main.rs)
    is_test: bool,
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
    pub fn new(db_client: &Client, is_test: &bool) -> Self {
        Self {
            db_client: db_client.clone(),
            job_scheduler: None, 
            is_test: *is_test,
        } 
    }

    async fn notify_relatives(res_id: &str, db_client: &Client) -> Result<bool, std::io::ErrorKind> {
        println!("sending phone number!");
        let collection = db_client.database(shared_struct::USERS).collection::<shared_struct::UsersLoggedInformation>(shared_struct::INFO);
        let filter = doc! {
            "res_ids": {
                "$in": [res_id]
            }
        };
        // Collect ALL relatives that has has this res_id
        let cursor = collection.find(filter).await;
        let results = match cursor {
            Ok(mut cursor) => {
                let mut results = Vec::new();
                while let Some(item) = cursor.next().await {
                    match item {
                        Ok(logged_info) => results.push(logged_info),
                        Err(err) => {
                            eprintln!("Error processing cursor item: {:?}", err);
                            return Err(std::io::ErrorKind::InvalidInput);
                        }
                    }
                }
                results
            },
            Err(err) => {
                eprintln!("Error querying user information: {:?}", err);
                return Err(std::io::ErrorKind::InvalidInput);
            }
        };
        println!("results: {:?}", results);
        for _info in results {
            println!("sending res_id: {} to number: {}", res_id, _info.phone_number.clone());
            let to_number = _info.phone_number;
            dotenv().ok();
            let client = reqwest::Client::new();
            let auth_token = env::var("AUTH_TOKEN").unwrap_or_default();
            let account_sid = env::var("ACCOUNT_SID").unwrap_or_default();
            print!("sid: {}", account_sid);
            let from_number = env::var("FROM_NUMBER").unwrap_or_default();
            let message = format!("Hello from KitchenGuardServer!, resident {} is in critical mode!", res_id);
            let url = format!("{}{}/Messages.json", shared_struct::SMS_SERVICE, account_sid);
            println!("url: {}", url);
            let params = [
                ("To", to_number),
                ("From", from_number),
                ("Body", message),
            ];
            println!("It has been sent!");

            let response = client.post(&url)
                .basic_auth(account_sid, Some(auth_token))
                .form(&params)
                .send()
                .await;
            match response {
                Ok(resp) => {
                    println!("Response: {:?}", resp.text().await.unwrap());
                }
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                }
            }
        }
        Ok(true)
    }


    fn alarm_duration_from_state(new_state: &shared_struct::States, is_test: &bool) -> Instant {
        if *is_test {println!("we are in tests");}
        else { println!("not in tests...");}
        match new_state {
            shared_struct::States::Unattended => Instant::now() + if *is_test { Duration::from_secs(10) } else { Duration::from_secs(20*60) }, // 20 minutes
            shared_struct::States::Alarmed => Instant::now() + if *is_test { Duration::from_secs(15) } else { Duration::from_secs(2*60) }, // 2 minutes
            shared_struct::States::CriticallyAlarmed => Instant::now() + if *is_test { Duration::from_secs(20) } else { Duration::from_secs(8*60)}, // 8 minutes
            _ => panic!("You should not give state '{:?}' to this function!", new_state),
        }
    }

    fn determine_task(old_state: &shared_struct::States, new_state: &shared_struct::States, res_id: &str, is_test: &bool) -> Option<TaskValue> {
        println!("is new state > old state? {:?}", new_state > old_state);
        if new_state > old_state { // this sometimes means timing an alarm
            match new_state {
                shared_struct::States::Unattended | shared_struct::States::Alarmed | shared_struct::States::CriticallyAlarmed => 
                    return Some(TaskValue {
                        type_of_task: TypeOfTask::NewTask,
                        scheduled_task: Some(ScheduledTask {
                            res_id: res_id.to_string(),
                            execute_at: StateHandler::alarm_duration_from_state(new_state, &is_test)
                        }),
                        res_id: res_id.to_string()
                    }),
                    _ => return None
            }
        } else {
            // this sometimes means cancelling an alarm
            match old_state {
                shared_struct::States::Unattended | shared_struct::States::Alarmed | shared_struct::States::CriticallyAlarmed => 
                    return Some(TaskValue {
                        type_of_task: TypeOfTask::Cancellation,
                        scheduled_task: None,
                        res_id: res_id.to_string()
                    }),
                _ => return None
            }
        }
    }

    /// This function is our main business logic, determining what we should do given a state and an event happening.
    /// Returns: the new state for the resident, and maybe a new room string
    fn determine_new_state(state_log: StateLog, list_of_sensors: &shared_struct::SensorLookup, data: &shared_struct::Event) 
    -> (shared_struct::States, Option<String>) {
        let current_state = state_log.state.clone();
        println!("Current state: {:?}", state_log.state);
        println!("current mode: {:?} and sensor: {:?}", data.mode, data.device_model);
        // If the pi has not sent health check yet, then we do nothing
        if current_state == shared_struct::States::Initialization || current_state == shared_struct::States::Faulty {
            return (current_state.clone(), None)
        }
        // Determine if resident walked into a new room, such as to change current_room_pir
        println!("device model: {}", data.device_model);
        println!("old room was: {}", state_log.current_room_pir);
        let new_room_pir = if data.device_model.to_lowercase().contains("pir") && data.mode == "True" {
                println!("resident is changing room because mode is: {}", data.mode);
                Some(data.device_model.clone())
            } else {
                None
            };
        // IF were in any of these states, then we only check if it's kitchen PIR detecting motion
        let new_state = if current_state == shared_struct::States::CriticallyAlarmed 
                                || current_state == shared_struct::States::Alarmed
                                || current_state == shared_struct::States::Unattended 
            {
            // if event is elderly moving into kitchen, then turn off alarm
            if data.device_model == list_of_sensors.kitchen_pir && data.mode == "True" { // occupancy: true
                // then go into Standby/Stove-attended according to state
                match current_state {
                    shared_struct::States::CriticallyAlarmed => shared_struct::States::Standby,
                    shared_struct::States::Alarmed => shared_struct::States::Attended,
                    shared_struct::States::Unattended => shared_struct::States::Attended,
                    _ => {
                        eprintln!("Unexpected state encountered: {:?}", current_state);
                        panic!("Invalid state transition detected");
                    }
                }
            } else if data.device_model == shared_struct::JOBSSCHEDULER_ID {
                println!("device found to be from jobscheduler");
                // An event from jobscheduler means that a timer was done, so give the appropriate task duration
                let next_state = match current_state {
                    shared_struct::States::CriticallyAlarmed => shared_struct::States::CriticallyAlarmed,
                    shared_struct::States::Alarmed => shared_struct::States::CriticallyAlarmed,
                    shared_struct::States::Unattended => shared_struct::States::Alarmed,
                    _ => {
                        eprintln!("Unexpected state encountered: {:?}", current_state);
                        panic!("Invalid state transition detected");
                    }
                };
                next_state
            } else if data.device_model == "USER" {
                // user pressed 'turn off alarm' inside website
                let next_state = match current_state {
                    // stove is turned off, if state is critical, so it goes to standby
                    shared_struct::States::CriticallyAlarmed => shared_struct::States::Standby,
                    _ => shared_struct::States::Unattended,
                };
                next_state
            } else {
                // if it's not the user moving into kitchen, don't do anything
                current_state.clone()
            }
        }
        // In attended, we check both kitchen PIR status and power plug status
        else if current_state == shared_struct::States::Attended 
        {
            if data.device_model == list_of_sensors.kitchen_pir && data.mode == "False" { // occupancy: false
                println!("resident is out of kitchen!");
                shared_struct::States::Unattended
                // If elderly turns off the stove
            } else if data.device_model == list_of_sensors.power_plug && data.mode == "OFF" { // power: Off
                shared_struct::States::Standby
            } else {
                // else do nothing, could be other room PIR saying somethin
                current_state.clone()
            }
        // If power plug gets turned on
        } 

        else if current_state == shared_struct::States::Standby 
        {
            println!("we are in standby!!!");
            if data.device_model == list_of_sensors.power_plug && data.mode == "ON" {
                shared_struct::States::Attended
            } else {
                current_state.clone()
            }
            // Should not be possible, but just in case
        }
        else 
        {
            // default to current_state
            current_state.clone()
        };
        return (new_state, new_room_pir)
    }

    async fn get_state(res_id: &str, db_client: &Client) -> Result<Option<StateLog>, mongodb::error::Error> {
        println!("Trying to find {}", res_id);
        db_client
            .database(shared_struct::RESIDENT_DATA)
            .collection::<StateLog>(shared_struct::STATES)
            .find_one(doc! {"res_id": &res_id})
            .sort(doc!{"_id": -1}) //finds the latest (datewise) entry matching res_id
            .await 
    }

    async fn get_sensors(res_id: &str, db_client: &Client) -> Result<Option<SensorLookup>, mongodb::error::Error> {
        db_client
            .database(shared_struct::RESIDENT_DATA)
            .collection::<shared_struct::SensorLookup>(shared_struct::SENSOR_LOOKUP)
            .find_one(doc! {"res_id": &res_id})
            .sort(doc!{"_id": -1})
            .await 
    }

    pub async fn create_user(username: &str, password: &str, phone_number: &str, db_client: &Client) 
    -> Result<Option<shared_struct::UsersLoggedInformation>, mongodb::error::Error> {
        // checks whether phone number contains any letters not a numner
        if !phone_number.chars().all(|letter| letter.is_numeric()) {
            return Err(std::io::ErrorKind::InvalidInput.into());
        }

        let user_salt = username.as_bytes().to_vec();
        let hashed_password = shared_struct::hash_password(password, &user_salt);
        println!("creating user");
        // The binary type is used, since it is more appropriate for storing bytes in mongodb
        let salt_binary = bson::Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: user_salt.clone(),
        };
        let user_fields = doc! {
            "$set": {
                "username": username,
                "password": hashed_password,
                "salt": salt_binary,
                "res_ids": Vec::<String>::new(),
                "phone_number": phone_number.to_string(),
            }
        };
        StateHandler::create_or_update_entry::<shared_struct::UsersLoggedInformation>("username", 
            username, user_fields, shared_struct::USERS, shared_struct::INFO, &db_client).await
    }

    // uses the T type, to allow for different collections to be found
    async fn create_or_update_entry<T>(identifier: &str, search_query: &str, data: bson::Document, database: 
        &str, collection: &str, db_client: &Client) 
    -> Result<Option<T>, mongodb::error::Error> 
    where 
    T: std::marker::Send + Sync + serde::de::DeserializeOwned + serde::Serialize + std::fmt::Debug + 'static
    {
        let collection_t = db_client.database(database).collection::<T>(collection);
        let filter = doc! { identifier: search_query};

        let res = collection_t
            .find_one_and_update(filter, data)
            .upsert(true)
            .return_document(mongodb::options::ReturnDocument::After)
            .await;
        res
    }

    /// Allows a user to see information on a resident
    pub async fn add_res_to_user(res_id: &str, username: &str, db_client: &Client) 
    -> Result<Option<shared_struct::UsersLoggedInformation>, mongodb::error::Error> {
        println!("Adding residents to user!");

        let update = doc! {
            "$addToSet": {
                "res_ids": res_id.to_string()
            }
        };
        
        StateHandler::create_or_update_entry("username", username, update, 
            shared_struct::USERS, shared_struct::INFO, db_client).await
    }

    /// the first step in connecting pi to server
    /// Initializes state
    async fn initialize_new_pi(data: &shared_struct::InitState, db_client: &Client) -> Result<(), std::io::ErrorKind> {
        let res_id = data.info.res_id.clone();
        let update = doc! {
            "$set": {
                "kitchen_pir": data.info.kitchen_pir.clone(),
                "power_plug": data.info.power_plug.clone(),
                "other_pir": data.info.other_pir.clone(),
                "led": data.info.led.clone(),
            }
        };
        // update sensors
        if let Err(err) = StateHandler::create_or_update_entry::<shared_struct::SensorLookup>("res_id", &data.info.res_id.clone(), update, shared_struct::RESIDENT_DATA, shared_struct::SENSOR_LOOKUP, &db_client).await {
            eprintln!("Failure in setting sensor lookup: {:?}", err);
        }
        let ip_update = doc! {
            "$set": {
                "res_ip": data.ip_addr.clone(),
                "res_id": &res_id,
            }
        };
        // update ip address of pi
        if let Err(err) = StateHandler::create_or_update_entry::<shared_struct::IpCollection>("res_id", &data.info.res_id.clone(), ip_update, shared_struct::RESIDENT_DATA, shared_struct::IP_ADDRESSES, &db_client).await {
            eprintln!("Failure in setting controller ip address: {:?}", err);
        }
        let state_log = StateLog {
            res_id: res_id.clone(),
            timestamp: chrono::Utc::now(),
            state: shared_struct::States::Initialization,
            current_room_pir: data.info.kitchen_pir.clone(),
            context: format!("{:?}", data.clone()),
        };
        if let Err(err) =  db_client.database(shared_struct::RESIDENT_DATA)
                                            .collection::<StateLog>(shared_struct::STATES)
                                            .insert_one(state_log).await 
        {
            eprintln!("Failed to save new state: {:?}", err);
            return Err(std::io::ErrorKind::InvalidInput);
        };
        let init_state = shared_struct::States::Initialization;
        PiCommunicator::send_new_state(&res_id, &init_state, 
            &data.info.kitchen_pir, &db_client).await;
        Ok(())
    }

    async fn handle_new_event(
        data: shared_struct::Event, 
        is_test: bool, 
        job_scheduler: Addr<JobsScheduler>, 
        db_client: &Client) 
        -> Result<shared_struct::States, std::io::ErrorKind> 
    {
        let res_id = data.res_id.to_string();
        // Log the event data
        let collection = db_client.database(shared_struct::RESIDENT_DATA)
            .collection::<shared_struct::Event>(shared_struct::RESIDENT_LOGS);
        if let Err(err) = collection.insert_one(&data).await {
            eprintln!("Failed to log event: {:?}", err);
            return Err(std::io::ErrorKind::InvalidData);
        }
        // Get current state and sensors
        // This shouldn't be able to fail, and if it does then the server should panic. So this is okay
        let latest_statelog = StateHandler::get_state(&res_id, &db_client).await.unwrap().unwrap();
        let sensor_lookup = StateHandler::get_sensors(&res_id, &db_client).await.unwrap().unwrap();
        let current_state = latest_statelog.state.clone();

        // determine the new state
        let (new_state, room_pir) = StateHandler::determine_new_state(latest_statelog.clone(), &sensor_lookup, &data);
        println!("new state found to be: {:?} with old state: {:?}", new_state, current_state);

        if new_state == shared_struct::States::CriticallyAlarmed && new_state == current_state && data.device_model == shared_struct::JOBSSCHEDULER_ID {
            println!(" SENDING SMS TO RELATIVES");
            // If we are currently in CriticallyAlarmed, then 8 minutes has passed and R6 describes 
            // notifiying the relatives now
            match StateHandler::notify_relatives(&res_id, &db_client).await {
                Ok(_) => println!("Correctly send sms to relatives!"),
                Err(err) => eprintln!("Error occured whilst sending sms to relatives: {:?}", err)
            };
        }
        let has_changed_state = new_state != current_state;
        if !has_changed_state && !room_pir.is_some() {
            println!("-------------SAME STATE");
            return Ok(new_state)
        }
        println!("changing state, {}", has_changed_state);
        let new_room_pir = match room_pir {
            Some(room) => room,
            None => latest_statelog.current_room_pir,
        };
        // Save the new state
        let state_log = StateLog {
            res_id: res_id.clone(),
            timestamp: chrono::Utc::now(),
            state: new_state.clone(),
            current_room_pir: new_room_pir.clone(),
            context: format!("{:?}", data),
        };
        let state_collection = db_client.database(shared_struct::RESIDENT_DATA)
            .collection::<StateLog>(shared_struct::STATES);
        if let Err(err) = state_collection.insert_one(state_log).await {
            eprintln!("Failed to save new state: {:?}", err);
            return Err(std::io::ErrorKind::InvalidInput);
        };
        
        if has_changed_state{ 
            // If the state changed, then send it to controller and determine whether or not we should start a timer
            PiCommunicator::send_new_state(&res_id, &new_state, &new_room_pir,  &db_client).await; 
            let task_type = StateHandler::determine_task(&current_state, &new_state, &res_id, &is_test);
            if task_type.is_some(){
                let task_el = task_type.unwrap();
                if task_el.type_of_task == TypeOfTask::NewTask {
                    println!("Sending task to scheduler!");
                match task_el.scheduled_task {
                        Some(task) => job_scheduler.do_send(task),
                        _=> eprint!("Tried to queue task, but was missing it")
                    }
                } else if task_el.type_of_task == TypeOfTask::Cancellation {
                    println!("cancelling timer in jobsscheduler!");
                    let res_id = task_el.res_id;
                    job_scheduler.do_send(CancelTask {
                        res_id: res_id.clone(),
                    });
                    if data.device_model == "USER" {
                        // When the user clicks 'turn off alarm', then the timer to go into critically should be cancelled, but a 
                        // new timer for being in unattended should be set also 
                        job_scheduler.do_send( ScheduledTask {
                            res_id,
                            execute_at: StateHandler::alarm_duration_from_state(&new_state, &is_test)
                        })
                    }
                }
            }
        }
        Ok(new_state)
    }

    /// Simply logs the health check
    async fn save_health_check(data: &shared_struct::HealthCheck, db_client: &Client) -> bool {
        // HealthCheck is a vector of tuples. Compare each entries first el to a sensor in SensorLookup
        if let Err(err) = db_client.database(shared_struct::RESIDENT_DATA)
            .collection::<shared_struct::HealthCheck>(shared_struct::DEVICE_HEALTH)
            .insert_one(data.clone()).await {
                eprint!("Failed to save health check: {:?}", err);
        }
        // checks each entry, if any aren't okay then the system is faulty
        data.data.iter().all(|sens | sens.1 == "ok")
    }
}

impl Handler<HealthCheck> for StateHandler {
    type Result = ();

    fn handle(&mut self, data: HealthCheck, _ctx: &mut Self::Context) -> Self::Result {
        let db_client = self.db_client.clone();
        actix::spawn(async move {
            let res_id = data.res_id.clone();
            let system_okay = StateHandler::save_health_check(&data, &db_client).await;
            let current_state = StateHandler::get_state(&res_id, &db_client).await.unwrap().unwrap();
            println!("in healthcheck: {} \n data:  {:?}",system_okay, data);
            
            // If we're in faulty, but system is okay again, then go to standby
            // OR If pi sent a healthy check, then that pi system is initalized and should go to standby
            if system_okay && (current_state.state == shared_struct::States::Initialization 
                            || current_state.state == shared_struct::States::Faulty) {
                let new_state = shared_struct::States::Standby;
                println!("INITIALIZED!!  GOING INTO STANDBY");
                let initialization_event = shared_struct::Event {
                    time_stamp: chrono::Utc::now().to_rfc3339(),
                    mode: "OK".to_string(),
                    event_data: "A healthy check was given".to_string(),
                    event_type_enum: "INIT".to_string(),
                    res_id: res_id.clone(),
                    device_model: shared_struct::STATEHANDLER_ID.to_string(),
                    device_vendor: "KG2".to_string(),
                    gateway_id: 1,
                    id: shared_struct::STATEHANDLER_ID.to_string()
                };
                let collection = db_client.database(shared_struct::RESIDENT_DATA).collection::<shared_struct::Event>(shared_struct::RESIDENT_LOGS);
                if let Err(err) = collection.insert_one(initialization_event).await {
                    eprintln!("Failed to log event: {:?}", err);
                    return;
                }
                let state_log = StateLog {
                    res_id: res_id.clone(),
                    timestamp: chrono::Utc::now(),
                    state: new_state.clone(),
                    current_room_pir: current_state.current_room_pir.clone(),
                    context: "system is now initialized".to_string(),
                };
                 let state_collection = db_client.database(shared_struct::RESIDENT_DATA).collection::<StateLog>(shared_struct::STATES);
                if let Err(err) = state_collection.insert_one(state_log).await {
                    eprintln!("Failed to save new state: {:?}", err);
                    return;
                };
                PiCommunicator::send_new_state(&res_id, &new_state, &current_state.current_room_pir,  &db_client).await;
            } else if !system_okay {
            // But if system is not okay whilst 
                println!("######  SYSTEM IS FAULTY");
                let new_state = shared_struct::States::Faulty;
                // if the system is not okay, then we go into faulty mode
                let faulty_event = shared_struct::Event {
                    time_stamp: chrono::Utc::now().to_rfc3339(),
                    mode: "FAULTY".to_string(),
                    event_data: "some sensor was faulty, look in db".to_string(),
                    event_type_enum: "FAULTY".to_string(),
                    res_id: res_id.clone(),
                    device_model: shared_struct::STATEHANDLER_ID.to_string(),
                    device_vendor: "KG2".to_string(),
                    gateway_id: 1,
                    id: shared_struct::STATEHANDLER_ID.to_string()
                };
                let collection = db_client.database(shared_struct::RESIDENT_DATA)
                    .collection::<shared_struct::Event>(shared_struct::RESIDENT_LOGS);
                if let Err(err) = collection.insert_one(faulty_event).await {
                    eprintln!("Failed to log event: {:?}", err);
                    return;
                }
                let state_log = StateLog {
                    res_id: res_id.clone(),
                    timestamp: chrono::Utc::now(),
                    state: new_state.clone(),
                    current_room_pir: current_state.current_room_pir.clone(),
                    context: "A sensor was faulty".to_string(),
                };
                 let state_collection = db_client.database(shared_struct::RESIDENT_DATA).collection::<StateLog>(shared_struct::STATES);
                if let Err(err) = state_collection.insert_one(state_log).await {
                    eprintln!("Failed to save new state: {:?}", err);
                    return;
                };
                PiCommunicator::send_new_state(&res_id, &new_state, &current_state.current_room_pir, &db_client).await;
            }
        });
    }
}

/// There is no endpoint for this function, since it will only be used for testing
impl Handler<AddRelative> for StateHandler {
    type Result = ();

    fn handle(&mut self, data: AddRelative, _ctx: &mut Self::Context) -> Self::Result {
        let res_id_to_add = data.res_id;
        let username = data.username;
        println!("Adding res_id: {} to user {}", res_id_to_add, username);
        let db_client = self.db_client.clone();
        // spawn a new thread to handle db interaction (which is async, and Handler doesn't allow for async operation)
        actix::spawn(async move {
            if let Err(err) = StateHandler::add_res_to_user(&res_id_to_add, &username, &db_client).await {
                eprintln!("Failed to add res_id to user, error: {:?}", err);
            }
        });
        ()
    }
}

/// When starting up the resident's system, we need to initialise some state (we start in standby), and set the ip address which
/// we send updates to. (unsafe because there is no authentication here)
impl Handler<shared_struct::InitState> for StateHandler {
    type Result = ();

    fn handle(&mut self, data: shared_struct::InitState, _ctx: &mut Self::Context ) -> Self::Result {
        println!("creating resident");
        let db_client = self.db_client.clone();
        actix::spawn(async move {
            StateHandler::initialize_new_pi(&data, &db_client).await.unwrap();
        });
    }
}

impl Handler<shared_struct::CreateUser> for StateHandler {
    type Result = ResponseFuture<Result<UsersLoggedInformation, std::io::Error>>;

    fn handle(&mut self, data: shared_struct::CreateUser, _ctx: &mut Self::Context ) -> Self::Result {
        println!("creating user");
        let db_client = self.db_client.clone();
        Box::pin(async move {
            match StateHandler::create_user(&data.username, &data.password, &data.phone_number, &db_client).await {
                Ok(Some(user)) => Ok(user),
                Ok(None) => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "User not created")),
                Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Database error: {:?}", e))),
            }
        })
    }
}

/// Handles what to do, when a sensor event comes from the Pi
/// It does way too much in one function, but the actor framework 
/// Returns a future that should be awaited
impl Handler<shared_struct::Event> for StateHandler {
    type Result = ResponseFuture<Result<shared_struct::States, std::io::ErrorKind>>;

    fn handle(&mut self, data: shared_struct::Event, _ctx: &mut Self::Context) -> Self::Result {
        println!("Caught an event for res_id: {:?}", data.res_id);
        let db_client = self.db_client.clone();
        let job_scheduler = self.job_scheduler.clone().unwrap();
        let is_test = self.is_test.clone();

        // Returns a future, that should be awaited
        Box::pin(async move {
            StateHandler::handle_new_event(data, is_test, job_scheduler, &db_client).await
        })
    }
}

// ====== TESTING ======
#[cfg(test)]
mod tests {
	use chrono::Utc;
    use super::*;

    #[test]
    fn test_determine_new_state_critically_alarmed_to_standby() {
        let current_state = shared_struct::States::CriticallyAlarmed;
        let current_state_log = StateLog {
            res_id: "1".to_string(),
            timestamp: Utc::now(),
            state: current_state.clone(),
            current_room_pir: "kitchen_pir_1".to_string(),
            context: "TEST".to_string(),
        };
        let list_of_sensors = shared_struct::SensorLookup {
            res_id: "1".to_string(),
            kitchen_pir: "kitchen_pir_1".to_string(),
            power_plug: "power_plug_1".to_string(),
            other_pir: vec![],
            led: vec![],
        };
        let data = shared_struct::Event {
            time_stamp: "2023-01-01T00:00:00Z".to_string(),
            mode: "True".to_string(),
            event_data: "".to_string(),
            event_type_enum: "".to_string(),
            res_id: "".to_string(),
            device_model: "kitchen_pir_1".to_string(),
            device_vendor: "".to_string(),
            gateway_id: 1,
            id: "".to_string(),
        };
        let is_test = true;
        let (new_state, _) = StateHandler::determine_new_state(current_state_log, &list_of_sensors, &data);
        let task = StateHandler::determine_task(&current_state, &new_state, "1", &is_test);

        assert_eq!(new_state, shared_struct::States::Standby);
        assert!(task.is_some());
        assert_eq!(task.unwrap().type_of_task, TypeOfTask::Cancellation);
    }

    #[test]
    fn test_determine_new_state_attended_to_unattended() {
        let current_state = shared_struct::States::Attended;
        let current_state_log = StateLog {
            res_id: "1".to_string(),
            timestamp: Utc::now(),
            state: current_state.clone(),
            current_room_pir: "kitchen_pir_1".to_string(),
            context: "TEST".to_string(),
        };
        let list_of_sensors = shared_struct::SensorLookup {
            res_id: "1".to_string(),
            kitchen_pir: "kitchen_pir_1".to_string(),
            power_plug: "power_plug_1".to_string(),
            other_pir: vec![],
            led: vec![],
        };
        let data = shared_struct::Event {
            time_stamp: "2023-01-01T00:00:00Z".to_string(),
            mode: "False".to_string(),
            event_data: "".to_string(),
            event_type_enum: "".to_string(),
            res_id: "".to_string(),
            device_model: "kitchen_pir_1".to_string(),
            device_vendor: "".to_string(),
            gateway_id: 1,
            id: "".to_string(),
        };
        let is_test = true;

        let (new_state, _) = StateHandler::determine_new_state(current_state_log, &list_of_sensors, &data);
        let task = StateHandler::determine_task(&current_state, &new_state, "1", &is_test);

        assert_eq!(new_state, shared_struct::States::Unattended);
        assert!(task.is_some());
        assert_eq!(task.unwrap().type_of_task, TypeOfTask::NewTask);
        // assert_eq!(task_value.type_of_task, TypeOfTask::NewTask);
        // assert!(task_value.scheduled_task.is_some());
    }

    #[test]
    fn test_determine_new_state_standby_to_attended() {
        let current_state = shared_struct::States::Standby;
        let current_state_log = StateLog {
            res_id: "1".to_string(),
            timestamp: Utc::now(),
            state: current_state,
            current_room_pir: "kitchen_pir_1".to_string(),
            context: "TEST".to_string(),
        };
        let list_of_sensors = shared_struct::SensorLookup {
            res_id: "1".to_string(),
            kitchen_pir: "kitchen_pir_1".to_string(),
            power_plug: "power_plug_1".to_string(),
            other_pir: vec![],
            led: vec![],
        };
        let data = shared_struct::Event {
            time_stamp: "2023-01-01T00:00:00Z".to_string(),
            mode: "ON".to_string(),
            event_data: "".to_string(),
            event_type_enum: "".to_string(),
            res_id: "".to_string(),
            device_model: "power_plug_1".to_string(),
            device_vendor: "".to_string(),
            gateway_id: 1,
            id: "".to_string(),
        };

        let (new_state, _) = StateHandler::determine_new_state(current_state_log, &list_of_sensors, &data);

        assert_eq!(new_state, shared_struct::States::Attended);
        // assert_eq!(task_value.type_of_task, TypeOfTask::None);
    }

    #[test]
    fn test_determine_new_state_no_change() {
        let current_state = shared_struct::States::Attended;
        let current_state_log = StateLog {
            res_id: "1".to_string(),
            timestamp: Utc::now(),
            state: current_state,
            current_room_pir: "kitchen_pir_1".to_string(),
            context: "TEST".to_string(),
        };
        let list_of_sensors = shared_struct::SensorLookup {
            res_id: "1".to_string(),
            kitchen_pir: "kitchen_pir_1".to_string(),
            power_plug: "power_plug_1".to_string(),
            other_pir: vec![],
            led: vec![],
        };
        let data = shared_struct::Event {
            time_stamp: "2023-01-01T00:00:00Z".to_string(),
            mode: "True".to_string(),
            event_data: "".to_string(),
            event_type_enum: "".to_string(),
            res_id: "".to_string(),
            device_model: "other_pir_1".to_string(),
            device_vendor: "".to_string(),
            gateway_id: 1,
            id: "".to_string(),
        };
        let (new_state, _) = StateHandler::determine_new_state(current_state_log, &list_of_sensors, &data);

        assert_eq!(new_state, shared_struct::States::Attended);
        // assert_eq!(task_value.type_of_task, TypeOfTask::None);
    }	

}