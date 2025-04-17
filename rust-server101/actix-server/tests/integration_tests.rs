


#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::Client;
    use kitchen_guard_server::classes::job_scheduler::{JobsScheduler, ScheduledTask, StartChecking};
    use kitchen_guard_server::classes::state_handler::{StateHandler, SetJobScheduler, Event};
    

    #[test]
    fn test_handler_sending_task() {
        let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());

        // let all workers use the client/Mongodb connection
        let db_client = Client::with_uri_str(uri).await.expect("failed to connect");
        // create_username_index(&client).await;

        // shows logging information when reaching server
        // Start state handler actor
        let state_handler = StateHandler {
            db_client: db_client.clone(),
            job_scheduler: None,
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

        // now send 2 messages, one saying powerplug is off, and then saying kitchen_pir occupancy is false
        let data1 = Event {
            time_stamp: "2023-01-01T00:00:00Z".to_string(),
            mode: "On".to_string(),
            event_data: "".to_string(),
            event_type_enum: "".to_string(),
            res_id: "".to_string(),
            device_model: "power_plug_1".to_string(),
            device_vendor: "".to_string(),
            gateway_id: 1,
            id: "".to_string(),
        };
        state_handler.do_send(data1);
        let data2 = Event {
            time_stamp: "2023-01-01T00:00:00Z".to_string(),
            mode: "false".to_string(),
            event_data: "".to_string(),
            event_type_enum: "".to_string(),
            res_id: "".to_string(),
            device_model: "kitchen_pir_1".to_string(),
            device_vendor: "".to_string(),
            gateway_id: 1,
            id: "".to_string(),
        };

        state_handler.do_send(data2);
        // now the job scheduler should have 1 job scheduled.
        

    }

    #[test]
    fn test_to_remove_alarm {

    }
}