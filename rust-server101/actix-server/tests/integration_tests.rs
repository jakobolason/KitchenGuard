


#[cfg(test)]
mod tests {
    use tokio;
    use actix::prelude::*;
    use mongodb::{bson::{oid::ObjectId, doc}, Client,};
    use kitchen_guard_server::classes::*;
    use kitchen_guard_server::classes::job_scheduler::{JobsScheduler, StartChecking, AmountOfJobs};
    use kitchen_guard_server::classes::state_handler::{StateHandler, SetJobScheduler, Event, StateLog, States};
    use serial_test::serial;
    use kitchen_guard_server::classes::shared_struct::{ScheduledTask, SensorLookup};
    use std::collections::VecDeque;

    #[tokio::test]
    #[serial]
    /// This test checks the functionality of the API by simulating a series of events and verifying the expected outcomes.
    async fn test_api() {
        let local = tokio::task::LocalSet::new();
        local.run_until(async {
            let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
            let db_client = Client::with_uri_str(uri).await.expect("failed to connect");

            let res_id = "test_resident_1";
             // Set up SensorLookup
            let sensor_collection = db_client.database("ResidentData").collection::<SensorLookup>("SensorLookup");
            
            // Create or update sensor lookup for test_resident_1
            let filter = doc! { "res_id": res_id };
            let test_sensor = SensorLookup {
                res_id: res_id.to_string(),
                kitchen_pir: "kitchen_pir_1".to_string(),
                power_plug: "power_plug_1".to_string(),
                other_pir: vec!["living_pir_1".to_string(), "bedroom_pir_1".to_string()],
                led: vec!["led_1".to_string(), "led_2".to_string()],
            };
            
            // Convert to document for upsert operation
            let mut sensor_doc = mongodb::bson::to_document(&test_sensor).expect("Failed to convert to document");
            sensor_doc.remove("_id"); // Remove _id field from the update document
            let update_doc = doc! { "$set": sensor_doc };
            
            // Upsert - update if exists, insert if doesn't exist
            sensor_collection.update_one(filter, update_doc).upsert(true).await
                .expect("Failed to upsert test sensor data");
            
            // Set up StateLog
            let state_collection = db_client.database("ResidentData").collection::<StateLog>("States");
            
            // Create or update state log for test_resident_1
            let filter = doc! { "res_id": res_id };
            let test_state = StateLog {
                _id: ObjectId::new(),
                res_id: res_id.to_string(),
                timestamp: chrono::Utc::now(),
                state: States::Standby, // Use your initial state
                context: "Initial test state".to_string(),
            };
            
            let mut state_doc = mongodb::bson::to_document(&test_state).expect("Failed to convert to document");
            state_doc.remove("_id");
            let update_doc = doc! { "$set": state_doc };
            
            state_collection.update_one(filter, update_doc).upsert(true).await
                .expect("Failed to upsert test state data");
    
            // Start state handler actor
            let state_handler: Addr<StateHandler> = StateHandler {
                db_client: db_client.clone(),
                job_scheduler: None,
                is_test: true
            }.start();
            
            // Start job scheduler actor and link to state handler
            let job_scheduler = JobsScheduler {
                tasks: VecDeque::<ScheduledTask>::new(),
                state_handler: state_handler.clone(),
            }.start();
            
            // Update state handler with job scheduler reference
            let _ = state_handler.send(SetJobScheduler {
                scheduler: Some(job_scheduler.clone()),
            }).await;
            // Start the scheduler's checking of tasks overdue
            let _ = job_scheduler.send(StartChecking).await;
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;// make sure scheduler is up and running
            let enter_kitchen = Event { // to make sure we're in Attended mode
                time_stamp: "2023-01-01T00:00:00Z".to_string(),
                mode: "True".to_string(),
                event_data: "".to_string(),
                event_type_enum: "".to_string(),
                res_id: res_id.to_string(),
                device_model: "kitchen_pir_1".to_string(),
                device_vendor: "".to_string(),
                gateway_id: 1,
                id: "".to_string(),
            };
            let _ = state_handler.send(enter_kitchen.clone()).await;
            let stove_off = Event {
                time_stamp: "2023-01-01T00:00:00Z".to_string(),
                mode: "OFF".to_string(),
                event_data: "".to_string(),
                event_type_enum: "".to_string(),
                res_id: res_id.to_string(),
                device_model: "power_plug_1".to_string(),
                device_vendor: "".to_string(),
                gateway_id: 1,
                id: "".to_string(),
            };
            let _ = state_handler.send(stove_off).await;
            // tokio::time::sleep(std::time::Duration::from_secs(3)).await;// make sure scheduler is up and running

            // now send 2 messages, one saying powerplug is on, and then saying kitchen_pir occupancy is false
            let stove_on = Event {
                time_stamp: "2023-01-01T00:00:00Z".to_string(),
                mode: "ON".to_string(),
                event_data: "".to_string(),
                event_type_enum: "".to_string(),
                res_id: res_id.to_string(),
                device_model: "power_plug_1".to_string(),
                device_vendor: "".to_string(),
                gateway_id: 1,
                id: "".to_string(),
            };
            let _ = state_handler.send(stove_on).await;
            
            println!("Send stove");
            // tokio::time::sleep(std::time::Duration::from_secs(1)).await;// make sure scheduler is up and running
    
            let leaving_kitchen = Event {
                time_stamp: "2023-01-01T00:00:00Z".to_string(),
                mode: "False".to_string(),
                event_data: "".to_string(),
                event_type_enum: "".to_string(),
                res_id: res_id.to_string(),
                device_model: "kitchen_pir_1".to_string(),
                device_vendor: "".to_string(),
                gateway_id: 1,
                id: "".to_string(),
            };
            let _ = state_handler.send(leaving_kitchen.clone()).await;
            println!("Send leaving kitchen");
            // tokio::time::sleep(std::time::Duration::from_secs(1)).await; // the actors are quite slow
    
            // now the job scheduler should have 1 job scheduled.
            let jobs_amount = job_scheduler.send(AmountOfJobs).await.unwrap();
            if let Ok(amount) = jobs_amount {
                assert_eq!(amount, 1)
            }
            // now send a message saying resident walked into kitchen again, so there should be no scheduled jobs
            let _ = state_handler.send(enter_kitchen).await;
            // tokio::time::sleep(std::time::Duration::from_secs(3)).await; // the actors are quite slow

            let new_jobs_amount = job_scheduler.send(AmountOfJobs).await.unwrap();
            
            if let Ok(amount) = new_jobs_amount {
                assert_eq!(amount, 0)
            }
            // now test db for the correct state
            let state_collection = db_client.database("ResidentData").collection::<StateLog>("States");
            match state_collection
                .find_one(doc! {"res_id": res_id})
                .sort(doc!{"_id": -1}) //finds the latest (datewise) entry matching "test_resident_1"
                .await
            {
                Ok(Some(document)) => assert_eq!(document.state, States::Attended),
                Ok(None) => panic!("No document found for res_id:res_id "),
                Err(err) => panic!("Error querying the database: {:?}", err),
            };
            // make user activate alarm
            let _ = state_handler.send(leaving_kitchen).await;
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;// the alarm is only 10secs when testing
            // now we should be alarmed
            match state_collection
                .find_one(doc! {"res_id": res_id})
                .sort(doc!{"_id": -1}) //finds the latest (datewise) entry matching res_id
                .await
            {
                Ok(Some(document)) => assert_eq!(document.state, States::Alarmed),
                Ok(None) => panic!("No document found for res_id:res_id"),
                Err(err) => panic!("Error querying the database: {:?}", err),
            };
        }).await;
    }

    #[tokio::test]
    #[serial]
    /// This test checks the functionality of the API by simulating a series of events and verifying the expected outcomes.
    async fn test_browser_responses() {
        let local = tokio::task::LocalSet::new();
        local.run_until(async {
            let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
            let db_client = Client::with_uri_str(uri).await.expect("failed to connect");
            println!("Hello");
            let web_handler = web_handler::WebHandler::new(
                cookie_manager::CookieManager::new(24), db_client.clone()
            ).start();
            // setup a basic user
            let username = "test_relative_1";
            let password = "123";
            let phone_number = "12345678";
            let res_id = "test_resident_1";
            let _ = StateHandler::create_user(username, password, phone_number, db_client.clone()).await;
            let _ = StateHandler::add_res_to_user(&res_id, &username, db_client.clone()).await;
            println!("created user");
            // tokio::time::sleep(std::time::Duration::from_secs(3)).await; // the actors are quite slow
            
            let cookie = web_handler.send(
                shared_struct::LoginInformation { username: username.to_string(), password: password.to_string() }
            ).await.unwrap();
            // assert!(cookie.is_some());
            let cookie_value = cookie.unwrap();
            assert!(!cookie_value.is_empty());


            // Make sure that there are events for the resident id given
            // Start state handler actor
            let state_handler: Addr<StateHandler> = StateHandler {
                db_client: db_client.clone(),
                job_scheduler: None,
                is_test: true
            }.start();
            
            // Start job scheduler actor and link to state handler
            let job_scheduler = JobsScheduler {
                tasks: VecDeque::<ScheduledTask>::new(),
                state_handler: state_handler.clone(),
            }.start();
            
            // Update state handler with job scheduler reference
            let _ = state_handler.send(SetJobScheduler {
                scheduler: Some(job_scheduler.clone()),
            }).await;
            // Start the scheduler's checking of tasks overdue
            let _ = job_scheduler.send(StartChecking).await;
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;// make sure scheduler is up and running
            let enter_kitchen = Event { // to make sure we're in Attended mode
                time_stamp: "2023-01-01T00:00:00Z".to_string(),
                mode: "True".to_string(),
                event_data: "".to_string(),
                event_type_enum: "".to_string(),
                res_id: res_id.to_string(),
                device_model: "kitchen_pir_1".to_string(),
                device_vendor: "".to_string(),
                gateway_id: 1,
                id: "".to_string(),
            };
            let _ = state_handler.send(enter_kitchen.clone()).await;
            let stove_off = Event {
                time_stamp: "2023-01-01T00:00:00Z".to_string(),
                mode: "OFF".to_string(),
                event_data: "".to_string(),
                event_type_enum: "".to_string(),
                res_id: res_id.to_string(),
                device_model: "power_plug_1".to_string(),
                device_vendor: "".to_string(),
                gateway_id: 1,
                id: "".to_string(),
            };
            let _ = state_handler.send(stove_off).await;
            // tokio::time::sleep(std::time::Duration::from_secs(3)).await;// make sure scheduler is up and running

            // now send 2 messages, one saying powerplug is on, and then saying kitchen_pir occupancy is false
            let stove_on = Event {
                time_stamp: "2023-01-01T00:00:00Z".to_string(),
                mode: "ON".to_string(),
                event_data: "".to_string(),
                event_type_enum: "".to_string(),
                res_id: res_id.to_string(),
                device_model: "power_plug_1".to_string(),
                device_vendor: "".to_string(),
                gateway_id: 1,
                id: "".to_string(),
            };
            let _ = state_handler.send(stove_on).await;
            
            println!("Send stove");

            let result = web_handler.send(
                shared_struct::ResIdFetcher { res_id: res_id.to_string() }
            ).await.unwrap();
            let result_value = result.unwrap();
            println!("Result: {:?}", result_value);
            assert!(!result_value.is_empty());

        }).await;
    }
}
