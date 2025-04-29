use std::collections::HashMap;
use std::time::{Duration, Instant};
use actix::{Message, Actor, Context, Handler};
use rand::{distr::Alphanumeric, Rng};

pub struct CookieEntry {
    pub res_ids: Vec<String>,
    pub lifetime: Instant
}
#[derive(Message)]
#[rtype(result="String")]
pub struct CreateNewCookie {
    pub res_uids: Vec<String>,
}

#[derive(Message)]
#[rtype(result = "Option<Vec<String>>")]
pub struct ValidateSession {
    pub token: String,
}

#[derive(Message)]
#[rtype(result = "bool")]
pub struct RemoveSession {
    pub token: String,
}

pub struct CookieManager {
    cookies: HashMap<String, CookieEntry>,
    session_duration: Duration, 
}
// cookies should also have some lifetime

impl CookieManager {
    pub fn new(session_duration_hours: u64) -> Self {
        CookieManager {
            cookies: HashMap::new(),
            session_duration: Duration::from_secs(session_duration_hours * 3600),
        }
    }

    pub fn check_cookie(cookies: &mut HashMap<String, CookieEntry>, cookie: String) -> bool {
        if let Some(entry) = cookies.get(&cookie) {
            if entry.lifetime < Instant::now() {
                cookies.remove(&cookie);
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn get_res_ids(cookies: &mut HashMap<String, CookieEntry>, cookie: String) -> Option<Vec<String>> {
        // Attempts to return the value, if the cookie exists
        if let Some(entry) = cookies.get(&cookie) {
            if entry.lifetime < Instant::now() {
                cookies.remove(&cookie);
                None
            } else {
                Some(entry.res_ids.clone())
            }
        } else {
            None
        }
    }

    fn generate_cookie() -> String {
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }
}


impl Actor for CookieManager {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Statehandler actor started!");
    }
}

impl Handler<CreateNewCookie> for CookieManager {
    type Result = String;

    fn handle(&mut self, list: CreateNewCookie, _ctx: &mut Self::Context) -> Self::Result {
        let new_cookie = CookieManager::generate_cookie();
        self.cookies.insert(new_cookie.clone(), CookieEntry { res_ids: list.res_uids, lifetime:  Instant::now() + self.session_duration});
        new_cookie
    }
}

impl Handler<ValidateSession> for CookieManager {
    type Result = Option<Vec<String>>;
    
    fn handle(&mut self, msg: ValidateSession, _: &mut Context<Self>) -> Self::Result {
        
        if let Some(entry) = CookieManager::get_res_ids(&mut self.cookies, msg.token) {
            return Some(entry)
        }
        None
    }
}

impl Handler<RemoveSession> for CookieManager {
    type Result = bool;
    
    fn handle(&mut self, msg: RemoveSession, _: &mut Context<Self>) -> Self::Result {
        self.cookies.remove(&msg.token).is_some()
    }
}
