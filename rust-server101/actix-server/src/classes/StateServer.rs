use std::collections::HashMap;

pub enum states {
    Standby,
    Attended,
    Unattended,
    Alarmed,
    CriticallyAlarmed
}

pub struct StateServer {
    state: states;
    sensor_lookup: HashMap; 
}

impl StateServer {
// Private functions
    // sends topic to turn off stove
    fn turn_off_stove() {

    }
    // sends topic to LED's and buzzer sound
    fn begin_alarm() {

    }
    // sends topic to powerplug to turn off, and
    fn critical_alarm() {

    }
    // stop alarm after elderly has returned to kitchen
    fn stop_alarm() {

    }
    // get database information on PIR sensor data
    fn check_users_room() {

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
    pub fn event(/** Should be a deserialized json*/) {

    }
// NOTE: Should maybe return both access token and a list of strings
//       for elder_uids
    pub fn check_credentials(username: str, password: str) -> str{

    }
}
