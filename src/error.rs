use thiserror::Error;

use crate::scheduler::Schedule;
use crate::simulation::JobID;

#[derive(Error, Debug)]
pub enum QMPError {
    #[error("Invalid job ID specified (job_id = {0})")]
    InvalidJobID(JobID),
    #[error("Invalid schedule (job_id = {job_id:?}, schedule = {schedule:?})")]
    InvalidSchedule { job_id: JobID, schedule: Schedule },
}

impl QMPError {
    pub fn invalid_job_id(job_id: JobID) -> anyhow::Error {
        QMPError::InvalidJobID(job_id).into()
    }

    pub fn invalid_schedule_error(job_id: JobID, schedule: Schedule) -> anyhow::Error {
        QMPError::InvalidSchedule { job_id, schedule }.into()
    }
}
