use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::program::Program;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    programs: Vec<Program>,
    job_requests: Vec<JobRequest>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(from = "(u64, usize)", into = "(u64, usize)")]
struct JobRequest {
    arrival_time: u64,
    program_id: usize,
}

impl From<(u64, usize)> for JobRequest {
    fn from(value: (u64, usize)) -> Self {
        JobRequest {
            arrival_time: value.0,
            program_id: value.1,
        }
    }
}

impl From<JobRequest> for (u64, usize) {
    fn from(value: JobRequest) -> Self {
        (value.arrival_time, value.program_id)
    }
}

impl JobRequest {
    pub fn arrival_time(&self) -> u64 {
        self.arrival_time
    }

    pub fn program_id(&self) -> usize {
        self.program_id
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct JobRequestRef<'a> {
    arrival_time: u64,
    program: &'a Program,
}

impl<'a> JobRequestRef<'a> {
    pub fn arrival_time(&self) -> u64 {
        self.arrival_time
    }

    pub fn program(&self) -> &'a Program {
        self.program
    }
}

impl Dataset {
    pub fn from_json_file(path: PathBuf) -> anyhow::Result<Dataset> {
        let json_str = std::fs::read_to_string(path)?;
        let mut dataset: Dataset = serde_json::from_str(&json_str)?;
        for program in &mut dataset.programs {
            program.build_internal_from_instructions()?;
        }
        dataset.validate()?;
        Ok(dataset)
    }

    pub fn get_request(&self, id: usize) -> JobRequestRef<'_> {
        let req = &self.job_requests[id];
        JobRequestRef {
            arrival_time: req.arrival_time(),
            program: &self.programs[req.program_id()],
        }
    }

    pub fn num_requests(&self) -> usize {
        self.job_requests.len()
    }

    pub fn requests(&self) -> impl Iterator<Item = JobRequestRef<'_>> {
        self.job_requests.iter().map(|req| JobRequestRef {
            arrival_time: req.arrival_time(),
            program: &self.programs[req.program_id()],
        })
    }

    fn validate(&self) -> anyhow::Result<()> {
        let max_id = self.programs.len();
        for req in &self.job_requests {
            if req.program_id >= max_id {
                anyhow::bail!("unknown program id in job_requests: {}", req.program_id);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Dataset;
    use std::path::PathBuf;

    #[test]
    fn test_parse_dataset_json() {
        let dataset =
            Dataset::from_json_file(PathBuf::from("samples/dataset.json")).expect("parse dataset");
        assert_eq!(dataset.num_requests(), 3);

        let req0 = dataset.get_request(0);
        assert_eq!(req0.arrival_time(), 0);
        let req1 = dataset.get_request(1);
        assert_eq!(req1.arrival_time(), 5);
        let program = req1.program();

        let instrs = program.instructions();
        assert_eq!(instrs.len(), 6);
        assert_eq!(instrs[0].latency(), 2);

        let req2 = dataset.get_request(2);
        assert_eq!(req2.arrival_time(), 10);
        assert_eq!(req2.program().instructions().len(), 3);
    }
}
