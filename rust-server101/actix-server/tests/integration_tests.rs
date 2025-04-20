


#[cfg(test)]
mod tests {
    use tokio;
    use actix::prelude::*;
    use mongodb::{bson::doc, Client,};
    use kitchen_guard_server::classes::job_scheduler::{JobsScheduler, ScheduledTask, StartChecking, AmountOfJobs};
    use kitchen_guard_server::classes::state_handler::{StateHandler, SetJobScheduler, Event, StateLog, States};
    use std::sync::{Arc, Mutex};
    use std::collections::VecDeque;

    #[tokio::test]
    async fn test_all_actors() {
        let local = tokio::task::LocalSet::new();
        local.run_until(async {
            let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
            let db_client = Client::with_uri_str(uri).await.expect("failed to connect");
    
            // Start state handler actor
            let state_handler: Addr<StateHandler> = StateHandler {
                db_client: db_client.clone(),
                job_scheduler: None,
                is_test: true
            }.start();
            
            // Start job scheduler actor and link to state handler
            let job_scheduler = JobsScheduler {
                tasks: Arc::new(Mutex::new(VecDeque::<ScheduledTask>::new())),
                state_handler: state_handler.clone(),
            }.start();
            
            // Update state handler with job scheduler reference
            state_handler.do_send(SetJobScheduler {
                scheduler: Some(job_scheduler.clone()),
            });
            // Start the scheduler's checking of tasks overdue
            job_scheduler.do_send(StartChecking);
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;// make sure scheduler is up and running
    
            // now send 2 messages, one saying powerplug is on, and then saying kitchen_pir occupancy is false
            let stove_on = Event {
                time_stamp: "2023-01-01T00:00:00Z".to_string(),
                mode: "On".to_string(),
                event_data: "".to_string(),
                event_type_enum: "".to_string(),
                res_id: "RES1".to_string(),
                device_model: "power_plug_1".to_string(),
                device_vendor: "".to_string(),
                gateway_id: 1,
                id: "".to_string(),
            };
            state_handler.do_send(stove_on);
            let enter_kitchen = Event { // to make sure we're in Attended mode
                time_stamp: "2023-01-01T00:00:00Z".to_string(),
                mode: "true".to_string(),
                event_data: "".to_string(),
                event_type_enum: "".to_string(),
                res_id: "RES1".to_string(),
                device_model: "kitchen_pir_1".to_string(),
                device_vendor: "".to_string(),
                gateway_id: 1,
                id: "".to_string(),
            };
            state_handler.do_send(enter_kitchen.clone());
            println!("Send stove");
    
            let leaving_kitchen = Event {
                time_stamp: "2023-01-01T00:00:00Z".to_string(),
                mode: "false".to_string(),
                event_data: "".to_string(),
                event_type_enum: "".to_string(),
                res_id: "RES1".to_string(),
                device_model: "kitchen_pir_1".to_string(),
                device_vendor: "".to_string(),
                gateway_id: 1,
                id: "".to_string(),
            };
            state_handler.do_send(leaving_kitchen.clone());
            println!("Send leaving kitchen");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await; // the actors are quite slow
    
            // now the job scheduler should have 1 job scheduled.
            let jobs_amount = job_scheduler.send(AmountOfJobs).await.unwrap();
            if let Ok(amount) = jobs_amount {
                assert_eq!(amount, 1)
            }
            // now send a message saying resident walked into kitchen again, so there should be no scheduled jobs
            state_handler.do_send(enter_kitchen);
            tokio::time::sleep(std::time::Duration::from_secs(1)).await; // the actors are quite slow

            let new_jobs_amount = job_scheduler.send(AmountOfJobs).await.unwrap();
            
            if let Ok(amount) = new_jobs_amount {
                assert_eq!(amount, 0)
            }
            // now test db for the correct state
            let state_collection = db_client.database("ResidentData").collection::<StateLog>("States");
            match state_collection
                .find_one(doc! {"res_id": "RES1"})
                .sort(doc!{"_id": -1}) //finds the latest (datewise) entry matching "RES1"
                .await
            {
                Ok(Some(document)) => assert_eq!(document.state, States::Attended),
                Ok(None) => panic!("No document found for res_id: RES1"),
                Err(err) => panic!("Error querying the database: {:?}", err),
            };
            // make user activate alarm
            state_handler.do_send(leaving_kitchen);
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;// the alarm is only 10secs when testing
            // now we should be alarmed
            match state_collection
                .find_one(doc! {"res_id": "RES1"})
                .sort(doc!{"_id": -1}) //finds the latest (datewise) entry matching "RES1"
                .await
            {
                Ok(Some(document)) => assert_eq!(document.state, States::Alarmed),
                Ok(None) => panic!("No document found for res_id: RES1"),
                Err(err) => panic!("Error querying the database: {:?}", err),
            };

        }).await;

        
        

    }

    // #[test]
    // fn test_to_remove_alarm {

    // }
}