use std::cmp::Ordering;

use crate::program::Program;

pub type JobID = u32;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Job {
    pub id: JobID,
    pub requested_time: u64,
    pub program: Program,
}

impl Job {
    pub fn new(id: JobID, requested_time: u64, program: Program) -> Self {
        Job {
            id,
            requested_time,
            program,
        }
    }
}

impl Ord for Job {
    fn cmp(&self, other: &Self) -> Ordering {
        other.requested_time.cmp(&self.requested_time)
    }
}

impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
