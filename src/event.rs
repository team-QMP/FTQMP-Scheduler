use crate::job::JobID;
use crate::scheduler::Schedule;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum EventType {
    ScheduleJob {
        job_id: JobID,
        schedule_time: u64,
    },
    /// Suspend program execution for a `duration` at time `t`.
    SuspendExec {
        duration: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Event {
    event_type: EventType,
    time: u64,
}

pub type EventList = BinaryHeap<Event>;

impl Event {
    pub fn schedule_job(job_id: JobID, schedule: Schedule, event_time: u64) -> Self {
        let event_type = EventType::ScheduleJob {
            job_id,
            schedule_time: schedule.z as u64,
        };
        Self {
            event_type,
            time: event_time,
        }
    }

    pub fn suspend_exec(time: u64, duration: u64) -> Self {
        Self {
            event_type: EventType::SuspendExec { duration },
            time,
        }
    }

    pub fn event_type(&self) -> &EventType {
        &self.event_type
    }

    pub fn event_time(&self) -> u64 {
        self.time
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        other.time.cmp(&self.time)
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
