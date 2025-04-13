use mongodb::Client;
use super::job_scheduler::JobsScheduler;
use super::state_handler::StateHandler;

pub struct AppState {
    pub state_handler: actix::Addr<StateHandler>,
    pub job_scheduler: actix::Addr<JobsScheduler>,
    pub db_client: Client,
}