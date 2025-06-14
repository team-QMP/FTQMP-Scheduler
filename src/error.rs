use thiserror::Error;

use crate::job::{Job, JobID};
use crate::scheduler::Schedule;

#[derive(Error, Debug)]
pub enum QMPError {
    #[error("Invalid job ID specified (job_id = {0})")]
    InvalidJobID(JobID),
    #[error("Invalid schedule (job_id = {}, schedule = {schedule:?})", job.id)]
    InvalidSchedule { job: Job, schedule: Schedule },
    #[error("Violate timing constraint")]
    ViolateTimingConstraint,
}

impl QMPError {
    pub fn invalid_job_id(job_id: JobID) -> anyhow::Error {
        QMPError::InvalidJobID(job_id).into()
    }

    pub fn invalid_schedule_error(job: Job, schedule: Schedule) -> anyhow::Error {
        QMPError::InvalidSchedule { job, schedule }.into()
    }
}
