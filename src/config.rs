use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::scheduler::SchedulerKind;
use crate::generator::GeneratorKind;

/// TODO: Support non-rectangle chip?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub size_x: u32,
    pub size_y: u32,
    pub micro_sec_per_cycle: u64,
    pub generator: GeneratorConfig,
    pub scheduler: SchedulerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub kind: SchedulerKind,
    pub time_limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    pub kind: GeneratorKind,
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
    use crate::generator::GeneratorKind;
    use crate::scheduler::SchedulerKind;
    use std::path::PathBuf;

    #[test]
    fn test_read_from_file() {
        let path = PathBuf::from("examples/test.toml"); // TODO
        let config = SimulationConfig::from_toml(path).unwrap();
        assert!(config.size_x == 6);
        assert!(config.size_y == 6);
        assert!(config.micro_sec_per_cycle == 100);
        assert!(config.scheduler.kind == SchedulerKind::Greedy);
        assert!(config.scheduler.time_limit == Some(60));
        assert!(config.generator.kind == GeneratorKind::Test);
    }
}
