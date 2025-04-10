use std::collections::HashMap;

#[derive(PartialEq, Eq)]
pub enum states {
    Standby,
    Attended,
    Unattended,
    Alarmed,
    CriticallyAlarmed
}

impl FromStr for states {

    type Err = ();

    fn from_str(input: &str) -> Result<states, Self::Err> {
        match input {
            "Standby"  => Ok(states::Standby),
            "Attended"  => Ok(states::Attended),
            "Unattended"  => Ok(states::Unattended),
            "Alarmed" => Ok(states::Alarmed),
            "CriticallyAlarmed" => Ok(states::CriticallyAlarmed),
            _      => Err(()),
        }
    }
}

pub enum sensorType {
    PIR,
    PP, // PowerPlug
    LED,
    SP, // speaker
}

// For every sensor event
struct SensorEvent {
    resId: str,
    timestamp: DateTime<Utc>,
    sensor_id: String,
    sensor_type: sensorType,
    value: str,              // could be state, or data. Maybe make it more general
}

// For when an alarm is sounded
struct AlertEvent {
    id: Uuid,
    timestamp: DateTime<Utc>,
    alert_type: AlertType,    // Alarmed or Critically alarmed
    state: AlertState,        // TRIGGERED, ACKNOWLEDGED, RESOLVED: Mutable, meaning it should be updated if alarm is turned off
    context: Json,            // Store full system state snapshot here
}

pub struct StateServer {
    state: states;
    sensor_lookup: HashMap; 
}


impl StateServer {
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
    fn notify_relatives() {

    }
    // make server listen to a topic
    fn sub(topic: str) {

    }
    // publish a topic(used in alarm e.g.)
    fn pub(topic: str) {
        
    }
    // This handles db connection and querying and response handling
    fn db_query(query: HashMap) { // don't know if hashmap is the best here

    }
    // check users given credentials
    fn check_credentials(user: str, pwd: str) {

    }
    // start a thread, that makes callbacks when 20 minutes ha spassed
    fn start_clock() {

    }

// Public functions
    pub fn event(sensor: sensors, data: Event, resId: str) {
        // check resId is a registered resident in db

        // log the event data
        let collection = client.database("Residents").collection(resId);
        let result = collection.insert_one(data.into_inner()).await;
        match result {
            Ok(_) => println!("Logged the event correctly"), 
            Err(err) => return false,
        }
        // Get state from db, maybe check the latest entry instead of only 1 entry
        let collection = client.database("States").collection(resId);
        let currentStateStr = collection.insert_one(
            doc! {"resId": resId};
        ).await?;

        println!("{:?}", currentStateStr);
        // Get the list of sensors for resident
        let collection = client.database("SensorLookup").collection(resId);
        let listOfSensors = collection.insert_one(
            doc! {"resId": resId};
        ).await?;

        // get state from string
        const currentState = states::from_str(currentStateStr).unwrap();
        // this if/else statement returns the new state
        const newState = 
            if currentState == states::CriticallyAlarmed || currentState == states::Alarmed
                                                         || currentState == states::Unattended 
            {
                // if event is elderly moving into kitchen, then turn off alarm
                if Event.device_model == listOfSensors.KitchenPIR && Event.mode == "true" { // occupancy: true
                    if currentState == states::Unattended || currentState == states::Alarmed {
                        // TODO: cancel jobscheduler timer given the resId
                    }
                    // then go into Standby/Stove-attended according to state
                    match currentState {
                        states::CriticallyAlarmed => states::Standby,
                        states::Alarmed => states::Attended,
                        states::Unattended => states::Attended,
                    }
                } else {
                    // if it's not the user moving into kitchen, don't do anything
                    currentState
                }
            } else if currentState == states::Attended {
                // if user is entering kitche, then cancel jobscheduler timer 
                if Event.device_model == listOfSensors.KitchenPIR && Event.mode == "false" { // occupancy: false
                    // TODO: then start 20min's timer in jobscheduler
                    states::Unattended
                } else if Event.device_model == listOfSensors.PowerPlug && Event.mode == "Off" {
                    states::Standby
                }
            } else if currentState == states::Standby {
                else if Event.device_model == listOfSensors.PowerPlug && Event.mode == "On" {
                    states::Attended
                }
            } else {
                // default to currentState
                currentState
            }
        }
        // now insert the new state
        // TODO: Determine whether we should update db, if it's the same state
        let collection = client.database("States").collection(resId);
        let result = collection.insert_one(newState.into_inner()).await;
        match result {
            Ok(_) => println!("Changed the state to {:?}", newState), 
            Err(err) => return false,
        }
    }


// NOTE: Should maybe return both access token and a list of strings
//       for elder_uids
    pub fn check_credentials(username: str, password: str) -> str{

    }
}

enum