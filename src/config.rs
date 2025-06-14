use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::preprocess::PreprocessKind;
use crate::scheduler::SchedulerKind;

/// TODO: Support non-rectangle chip?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub size_x: u32,
    pub size_y: u32,
    pub micro_sec_per_cycle: u64,
    #[serde(default)]
    pub no_output_program: bool,
    pub enable_defrag: bool,
    pub defrag_interval: Option<u64>,
    pub preprocessor: PreprocessorConfig,
    pub scheduler: SchedulerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub kind: SchedulerKind,
    pub time_limit: Option<u32>,
    pub batch_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessorConfig {
    pub processes: Vec<PreprocessKind>,
    pub num_cuboids: Option<u32>,
}

impl SimulationConfig {
    pub fn from_toml(path: PathBuf) -> Result<SimulationConfig> {
        let toml_str = std::fs::read_to_string(path)?;
        let config: SimulationConfig = toml::from_str(&toml_str)?;
        Ok(config)
    }
}

#[cfg(test)]
pub mod test {
    use crate::config::SimulationConfig;
    use crate::scheduler::SchedulerKind;
    use crate::test_utils;
    use std::path::PathBuf;

    #[test]
    fn test_read_from_file() {
        let path = PathBuf::from(test_utils::TEST_TOML_FILE);
        let config = SimulationConfig::from_toml(path).unwrap();
        assert!(config.size_x == 6);
        assert!(config.size_y == 6);
        assert!(config.micro_sec_per_cycle == 100);
        assert!(!config.enable_defrag);
        assert!(config.defrag_interval == Some(1000));
        assert!(config.scheduler.kind == SchedulerKind::Greedy);
        assert!(config.scheduler.time_limit == Some(60));
        assert!(config.scheduler.batch_size == Some(3));
    }
}
