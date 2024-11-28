pub mod greedy_scheduler;

use crate::ds::polycube::Polycube;

pub enum SchedulerKind {
    Greedy,
}

pub trait Scheduler {
    fn add_job(&mut self, job_id: u32, program: Polycube);
    fn run(&mut self) -> Vec<(u32, Polycube)>;
}
