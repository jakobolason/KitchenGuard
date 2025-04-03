use std::collections::HashMap;

pub enum states {
    Standby,
    Attended,
    Unattended,
    Alarmed,
    CriticallyAlarmed
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
    pub fn event(sensor: sensors, data: str, resId: str) {
        // check resId is a registered resident in db

        // log the event data
        let collection = client.database("Residents").collection(resId);
        let result = collection.insert_one(data.into_inner()).await;
        match result {
            Ok(_) => return true, 
            Err(err) => return false,
        }

        // If it was PIR sensor in kitchen setting occupancy:false, then check if powerplug is on

        // if it was kitchen PIR saying occupancy:true and alarm is on, then turn off alarm

    }


// NOTE: Should maybe return both access token and a list of strings
//       for elder_uids
    pub fn check_credentials(username: str, password: str) -> str{

    }
}

enum