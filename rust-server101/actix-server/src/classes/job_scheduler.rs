use actix::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use std::future::Future;
// use std::process::Output;
use std::collections::VecDeque;
use std::thread::sleep;

use super::state_handler::{StateHandler};

#[derive(Message)]
#[rtype(result = "()")]
struct CheckJobs;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct ScheduledTask {
  res_id: String,
  execute_at: Instant,
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

// the schedule function, so by sending a 'ScheduledTask' to this struct it is recieved here
impl Handler<ScheduledTask> for JobsScheduler {
	type Result = ();

	fn handle(&mut self, msg: ScheduledTask, _ctx: &mut Self::Context) {
		let task = ScheduledTask {
			res_id: msg.res_id,
			execute_at: msg.execute_at,
		};
		let mut tasks = self.tasks.lock().unwrap();
		// use a lambda function to find the place task should be emplaced
		let pos = tasks.iter().position(|t| t.execute_at > task.execute_at)
			.unwrap_or(tasks.len());
		tasks.insert(pos, task);
	}
}

impl Handler<CheckJobs> for JobsScheduler {
	type Result = ();

	fn handle(&mut self, _msg: CheckJobs, _ctx: &mut Self::Context) {
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
			println!("Do sometghin");
		} else if is_empty { // might not be neccessary
			return
		}
		else {
			// sleep(Duration::from_secs(10));
		}
	}
}

impl JobsScheduler {

	pub fn cancel(&self, res_id: String) -> bool {
		// use unwrap to check integrity of tasks.lock
		let mut tasks = self.tasks.lock().unwrap();
		if let Some(pos) = tasks.iter().position(|t| t.res_id == res_id) {
			tasks.remove(pos);
			true
		} else {
			false
		}
	}
	
	// pub fn start(self) -> impl Future<Output = ()> {
	// 	// make a clone of the address to 'tasks'
	// 	let tasks = Arc::clone(&self.tasks);
	// 	async move {
	// 		loop {
	// 			// Check if the front timer is expired
	// 			let (next_task, is_empty) = {
	// 				let mut queue = tasks.lock().unwrap();
	// 				if queue.is_empty() {
	// 					(None, true)
	// 				} else {
	// 					let now = Instant::now();
	// 					if queue[0].execute_at <= now {
	// 						(Some(queue.pop_front().unwrap()), false)
	// 					} else {
	// 						(None, false)
	// 					}
						
	// 				}
	// 			};
	// 			if let Some(task) = next_task {
	// 				println!("Do sometghin");
	// 			} else if is_empty { // might not be neccessary
	// 				break;
	// 			}
	// 			else {
	// 				sleep(Duration::from_secs(10));
	// 			}
	// 		}
	// 	}
	// }
}