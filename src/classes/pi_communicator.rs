use reqwest;
use mongodb::{Client, bson::doc};
use serde::Deserialize;

use super::shared_struct::{States, IpCollection, RESIDENT_DATA, IP_ADDRESSES, PI_LISTENER};

pub struct PiCommunicator;

impl PiCommunicator {
    // send the current state to the pi with the given res_id
    async fn _send_to_pi(pi_ip: String, new_state: States, current_room_pir: &str) {
        // let pi_ip = self.ips.get(&res_id);
        let ip = pi_ip;
        let url = format!("http://{}/{}", ip, PI_LISTENER);
        let client = reqwest::Client::new();
        match client.post(&url)
            .json(&serde_json::json!({
                "new_state": new_state,
                "current_room_pir": current_room_pir
            }))
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

    pub async fn send_new_state(res_id: String, new_state: States, current_room_pir: &str, db_client: Client) {
        // query db for the residents ip-address
        let ip_addr  = db_client.database(RESIDENT_DATA).collection::<IpCollection>(IP_ADDRESSES).find_one(doc! {"res_id": res_id}).await;
        match ip_addr {
            Ok(Some(logs)) => {
                // now send to pi
                println!("!!!!!Send to pi");
                PiCommunicator::_send_to_pi(logs.res_ip, new_state, current_room_pir).await;
            }
            Ok(None) => println!("an error occured whilst sending new state, no log"),
            Err(err) => println!("an error occured whilst sending new state: {:?}", err),
        };
    }
}
