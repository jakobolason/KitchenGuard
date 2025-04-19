use actix::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use std::collections::VecDeque;

use super::state_handler::{StateHandler, Event};

#[derive(Message)]
#[rtype(result = "()")]
struct CheckJobs;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct ScheduledTask {
  pub res_id: String,
  pub execute_at: Instant,
}
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct CancelTask {
	pub res_id: String
}

// impl Message for ScheduledTask {
// 	type Result = ();
// }
#[derive(Clone, Debug)]
pub struct JobsScheduler {
  pub tasks: Arc<Mutex<VecDeque<ScheduledTask>>>,
  pub state_handler: Addr<StateHandler>,
}

// Use actor for concurrency design
impl Actor for JobsScheduler {
	type Context = Context<Self>;

	fn started(&mut self, _ctx: &mut Self::Context) {
		println!("Job scheduler started!");
    }
}

// ====== Handlers for messages to JobsScheduler ======

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartChecking;
// Makes it start looking every 10 seconds for any expired tasks
impl Handler<StartChecking> for JobsScheduler {
	type Result = ();

	fn handle(&mut self, _msg: StartChecking, ctx: &mut Self::Context) {
		println!("Job scheduler have started checking");
		ctx.run_interval(Duration::from_secs(10), |_, ctx| {
            // Send the check jobs message to self
            ctx.address().do_send(CheckJobs);
        });
	}
}

impl Handler<CancelTask> for JobsScheduler {
	type Result = ();

	fn handle(&mut self, task: CancelTask, _ctx: &mut Self::Context) {
		self.cancel(task.res_id);
	}
}

// the schedule function, so by sending a 'ScheduledTask' to this struct it is recieved here
impl Handler<ScheduledTask> for JobsScheduler {
	type Result = ();

	fn handle(&mut self, msg: ScheduledTask, _ctx: &mut Self::Context) {
		println!("Recieved a task to schedule! duration: {:?}", msg.execute_at);
		self.schedule(msg);
	}
}

impl Handler<CheckJobs> for JobsScheduler {
	type Result = ();

	fn handle(&mut self, _msg: CheckJobs, _ctx: &mut Self::Context) {
		self.check_for_jobs();
	}
}

#[derive(Message)]
#[rtype(result = "Result<i8, std::io::Error>")] // there should only be a length of 1 when testing the length
pub struct AmountOfJobs;
impl Handler<AmountOfJobs> for JobsScheduler {
	type Result = Result<i8, std::io::Error>;

	fn handle(&mut self, _data: AmountOfJobs, _ctx: &mut Self::Context) -> Self::Result { // Self is referrring to AmountOfJobs struct here
		let tasks = self.tasks.lock().unwrap();
		return Ok(tasks.len() as i8);
	}
}

impl JobsScheduler {

	pub fn cancel(&self, res_id: String) -> bool {
		println!("Asked to cancel a timer!");
		// use unwrap to check integrity of tasks.lock
		let mut tasks = self.tasks.lock().unwrap();
		if let Some(pos) = tasks.iter().position(|t| t.res_id == res_id) {
			tasks.remove(pos);
			println!("successfully removed entry, remaining size: {}", tasks.len());
			true
		} else {
			println!("failed to removed entry, remaining size: {}", tasks.len());
			false
		}
	}

	pub fn schedule(&self, msg: ScheduledTask) {
		let mut tasks = self.tasks.lock().unwrap();
		// use a lambda function to find the place task should be emplaced
		let pos = tasks.iter().position(|t| t.execute_at > msg.execute_at)
			.unwrap_or(tasks.len());
		tasks.insert(pos, msg);
	}

	fn check_for_jobs(&self) {
		// Check if the front timer is expired
		let (next_task, is_empty) = {
			let mut queue = self.tasks.lock().unwrap();
			if queue.is_empty() {
				(None, true)
			} else {
				let now = Instant::now();
				if queue[0].execute_at <= now {
					(Some(queue.pop_front().unwrap()), false)
				} else {
					(None, false)
				}
				
			}
		};
		if let Some(task) = next_task {
			let scheduler_event = Event {
				time_stamp: chrono::Utc::now().to_rfc3339(),
				mode: "DONE".to_string(),
				event_data: "COMPLETED".to_string(),
				event_type_enum: "job_scheduler".to_string(),
				res_id: task.res_id,
				device_model: "JobScheduler".to_string(),
				device_vendor: "SELF".to_string(),
				gateway_id: 1,
				id: "1".to_string(),
			};
			self.state_handler.do_send(scheduler_event);
			println!("Should have been sent to state handler now");
		} else if is_empty { // might not be neccessary
			return
		}
		else {
			// sleep(Duration::from_secs(10));
		}
	}
}



// ====== TESTING ======
#[cfg(test)]
mod tests {
	use super::*;
	use super::StateHandler;
	use mongodb::Client;

	#[test]
	fn test_job_scheduler_initialization() {
		// call functions of the struct, not the actor
	}


}
