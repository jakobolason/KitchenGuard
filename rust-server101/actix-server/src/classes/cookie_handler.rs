use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::time::Instant;

pub struct CookieEntry {
    pub cookie: String,
    pub res_ids: Vec<String>,
    pub lifetime: Instant
}

pub struct CookieHandler {
    cookies: Arc<Mutex<VecDeque<CookieEntry>>>
}
// cookies should also have some lifetime

impl CookieHandler {
    pub fn check_cookie(self, cookie: String) -> bool {
        let mut cookies = self.cookies.lock().unwrap();
        if let Some(entry) = cookies.iter().find(|entry| entry.cookie == cookie) {
            if entry.lifetime < Instant::now() {
                // then remove the entry
                if let Some(pos) = cookies.iter().position(|entry| entry.cookie == cookie) {
                    cookies.remove(pos);
                }
                return false;
            }
            true
        } else {
            false
        }
    }

    pub fn get_res_ids(self, cookie: String) -> Vec<String> {
        // Attempts to return the value, if the cookie exists
        let cookies = self.cookies.lock().unwrap();
        if let Some(entry) = cookies.iter().find(|entry| *entry.cookie == cookie) {
            entry.res_ids.clone()
        } else {
            Vec::new()
        }
    }

    pub fn create_new_cookie(self, res_ids: Vec<String>) {
        // create a 
    }
}