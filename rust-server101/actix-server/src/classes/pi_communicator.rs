use std::{collections::{HashMap, VecDeque}, hash::Hash};
use reqwest;
use mongodb::{Client, bson::doc};
use serde::Deserialize;

use super::shared_struct::ScheduledTask;
use super::state_handler::States;
use super::shared_struct::{ResidentData, ip_addresses, pi_listener};

pub struct PiCommunicator;

#[derive(Clone, Deserialize)]
struct IpAddressLogs {
    res_id: String,
    res_ip: String,
}

impl PiCommunicator {
    // send the current state to the pi with the given res_id
    async fn _send_to_pi(pi_ip: String, new_state: States) {
        // let pi_ip = self.ips.get(&res_id);
        let ip = pi_ip;
        let url = format!("http://{}/{}", ip, pi_listener);
        let client = reqwest::Client::new();
        match client.post(&url)
            .json(&new_state)
            .send()
            .await {
        Ok(response) => {
            if response.status().is_success() {
            println!("State updated successfully for {}", ip.clone());
            } else {
            eprintln!("Failed to update state for {}: {}", ip.clone(), response.status());
            }
        }
        Err(err) => {
            eprintln!("Error sending request to {}: {}", ip, err);
        }
        }
    }

    pub async fn send_new_state(res_id: String, new_state: States, db_client: Client) {
        // query db for the residents ip-address
        let ip_collection  = db_client.database(ResidentData).collection::<IpAddressLogs>(ip_addresses);
        let ip_addr = ip_collection.find_one(doc! {"res_id": res_id}).await;
        match ip_addr {
            Ok(Some(logs)) => {
                // now send to pi
                PiCommunicator::_send_to_pi(logs.res_ip, new_state).await;
            }
            Ok(None) => println!("an error occured whilst sending new state"),
            Err(_) => println!("an error occured whilst sending new state"),
        };
    }
}
