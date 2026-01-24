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
        dataset.validate()?;
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

    fn validate(&self) -> anyhow::Result<()> {
        let max_id = self.programs.len();
        for (_, program_id) in &self.job_requests {
            if *program_id >= max_id {
                anyhow::bail!("unknown program id in job_requests: {program_id}");
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

        let (t0, _) = dataset.get_request(0);
        assert_eq!(t0, 0);
        let (t1, program) = dataset.get_request(1);
        assert_eq!(t1, 5);

        let instrs = program.instructions();
        assert_eq!(instrs.len(), 6);
        assert_eq!(instrs[0].latency(), 2);

        let (t2, program2) = dataset.get_request(2);
        assert_eq!(t2, 10);
        assert_eq!(program2.instructions().len(), 3);
    }
}
