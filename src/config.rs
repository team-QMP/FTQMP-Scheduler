use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use anyhow::Result;

/// TODO: Support non-rectangle chip?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub size_x: u32,
    pub size_y: u32,
    pub micro_sec_per_cycle: u128,
}

impl SimulationConfig {
    pub fn from_json_file(path: PathBuf) -> Result<SimulationConfig> {
        let json_str = std::fs::read_to_string(path)?;
        let config: SimulationConfig = serde_json::from_str(&json_str)?;
        Ok(config)
    }
}


#[cfg(test)]
pub mod test {
    use std::path::PathBuf;
    use crate::config::SimulationConfig;

    #[test]
    fn test_read_from_file() {
        let path = PathBuf::from("configs/test.json"); // TODO
        let config = SimulationConfig::from_json_file(path).unwrap();
        assert!(config.size_x == 6);
        assert!(config.size_y == 6);
        assert!(config.micro_sec_per_cycle == 100);
    }

}

