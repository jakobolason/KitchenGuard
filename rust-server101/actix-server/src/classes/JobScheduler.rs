extern crate timer;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use std::future::Future;
use std::process::Output;
use std::collections::VecDeque;
use std::thread::sleep;

// Define a type alias for the callback function
type CallbackFn = Box<dyn FnOnce() + Send + 'static>;

struct ScheduledTask {
  id: String,
  callback: CallbackFn,
  execute_at: Instant,
}

struct JobsSheduler {
  tasks: Arc<Mutex<VecDeque<ScheduledTask>>>,
}

impl JobsSheduler {
  // Constructor to be called in app instantiation (main.rs)
    fn new() -> Self {
      JobsSheduler {
        tasks: Arc::new(Mutex::new(VecDeque::<ScheduledTask>::new())),
      }
    }

	fn schedule<F>(&self, callback: F, delay: Duration, res_id: String)
	where
		// the callback function for when timer expires, should be called once, 'Send' gives owner
		// -ship from this thread to the StateServer, and 'static' means the functin should be static
		F: FnOnce() + Send + 'static,
	{
		let task = ScheduledTask {
			id: res_id,
			callback: Box::new(callback),
			execute_at: Instant::now() + delay,
		};
		let mut tasks = self.tasks.lock().unwrap();
		// use a lambda function to find the place task should be emplaced
		let pos = tasks.iter().position(|t| t.execute_at > task.execute_at)
			.unwrap_or(tasks.len());
		tasks.insert(pos, task);
	}

	fn cancel(&self, id: String) -> bool {
		// use unwrap to check integrity of tasks.lock
		let mut tasks = self.tasks.lock().unwrap();
		if let Some(pos) = tasks.iter().position(|t| t.id == id) {
			tasks.remove(pos);
			true
		} else {
			false
		}
	}
	
	fn start(self) -> impl Future<Output = ()> {
		// make a clone of the address to 'tasks'
		let tasks = Arc::clone(&self.tasks);
		async move {
			loop {
				// Check if the front timer is expired
				let (next_task, is_empty) = {
					let mut queue = tasks.lock().unwrap();
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
					((task.callback)());
				} else if is_empty { // might not be neccessary
					break;
				}
				else {
					sleep(Duration::from_secs(10));
				}
			}
		}
	}
}