use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::program::Program;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    programs: Vec<Program>,
    job_requests: Vec<(u64, usize)>,
}

impl Dataset {
    pub fn from_json_file(path: PathBuf) -> anyhow::Result<Dataset> {
        let json_str = std::fs::read_to_string(path)?;
        let dataset: Dataset = serde_json::from_str(&json_str)?;
        Ok(dataset)
    }

    pub fn get_request(&self, id: usize) -> (u64, &Program) {
        let (time, program_id) = self.job_requests[id];
        (time, &self.programs[program_id])
    }

    pub fn num_requests(&self) -> usize {
        self.job_requests.len()
    }

    pub fn requests(&self) -> Vec<(u64, &Program)> {
        self.job_requests
            .iter()
            .map(|&(t, id)| (t, &self.programs[id]))
            .collect()
    }
}
