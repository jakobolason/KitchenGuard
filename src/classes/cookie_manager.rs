use std::collections::HashMap;
use std::time::{Duration, Instant};
use rand::{distr::Alphanumeric, Rng};

pub struct CookieEntry {
    pub res_ids: Vec<String>,
    pub lifetime: Instant
}

pub struct CookieManager {
    cookies: HashMap<String, CookieEntry>,
    session_duration: Duration, 
}
// cookies should also have some lifetime

impl CookieManager {
    pub fn new(session_duration_hours: u64) -> Self {
        Self {
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

    pub fn create_new_cookie(&mut self, res_ids: Vec<String>) -> String {
        let new_cookie = CookieManager::generate_cookie();
        self.cookies.insert(new_cookie.clone(), CookieEntry { res_ids, lifetime:  Instant::now() + self.session_duration});
        new_cookie
    }

    pub fn validate_session(&mut self, cookie: String) -> Option<Vec<String>> {
        if let Some(entry) = CookieManager::get_res_ids(&mut self.cookies, cookie) {
            return Some(entry)
        }
        None
    }

    pub fn remove_session(&mut self, cookie: String) -> bool {
        self.cookies.remove(&cookie).is_some()
    }
}
