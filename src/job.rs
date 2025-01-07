use std::cmp::Ordering;

use crate::program::Program;

pub type JobID = u32;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Job {
    pub id: JobID,
    pub added_time: u128,
    pub program: Program,
}

impl Job {
    pub fn new(id: JobID, added_time: u128, program: Program) -> Self {
        Job {
            id,
            added_time,
            program,
        }
    }
}

impl Ord for Job {
    fn cmp(&self, other: &Self) -> Ordering {
        other.added_time.cmp(&self.added_time)
    }
}

impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
