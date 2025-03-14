use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::program::{Program, ProgramCounter, ProgramFormat};

pub type JobID = u32;

#[derive(Debug, Clone, Eq, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    // Requested, but not scheduled
    Waiting,
    // Scheduled, but not running
    Scheduled,
    Running,
    Finished,
}

// TODO: add program counter to each job (or program)
#[derive(Debug, Clone, Eq, Serialize, Deserialize, PartialEq)]
pub struct Job {
    pub id: JobID,
    pub requested_time: u64,
    pub program: Program,
    /// The time when the execution of this job will start.
    start_time: Option<u64>,
    status: JobStatus,
}

impl Job {
    pub fn new(id: JobID, requested_time: u64, program: Program) -> Self {
        Job {
            id,
            requested_time,
            program,
            start_time: None,
            status: JobStatus::Waiting,
        }
    }

    pub fn start_time(&self) -> Option<u64> {
        self.start_time
    }

    pub fn update_start_time(&mut self, stime: u64) {
        self.start_time = Some(stime)
    }

    pub fn total_execution_cycle(&self) -> ProgramCounter {
        match self.program.format() {
            ProgramFormat::Polycube(poly) => (poly.max_z() - poly.min_z() + 1) as ProgramCounter,
            ProgramFormat::Cuboid(cs) => {
                let (min_z, max_z) = cs.iter().fold(
                    (ProgramCounter::MAX, ProgramCounter::MIN),
                    |(min_z, max_z), c| {
                        (
                            ProgramCounter::min(min_z, c.pos().z as ProgramCounter),
                            ProgramCounter::max(max_z, c.pos().z as ProgramCounter),
                        )
                    },
                );
                max_z - min_z + 1
            }
        }
    }

    pub fn status(&self) -> &JobStatus {
        &self.status
    }

    pub fn update_status(&mut self, status: JobStatus) {
        self.status = status;
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
