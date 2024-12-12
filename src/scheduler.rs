pub mod greedy_scheduler;

use crate::simulation::JobID;
use crate::ds::polycube::Polycube;

#[derive(Debug, Clone, PartialEq)]
pub enum SchedulerKind {
    Greedy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Schedule {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub rotate: i32,
    pub flip: bool,
}

impl Schedule {
    pub fn new(x: i32, y: i32, z: i32, rotate: i32, flip: bool) -> Self {
        Self {
            x,
            y,
            z,
            rotate,
            flip
        }
    }
}

pub fn apply_schedule(program: &Polycube, schedule: &Schedule) -> Polycube {
    unimplemented!()
}

pub trait Scheduler {
    fn add_job(&mut self, job_id: JobID, program: Polycube);
    fn run(&mut self) -> Vec<(JobID, Schedule)>;
}
