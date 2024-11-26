use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use anyhow::Result;

/// TODO: Support non-rectangle chip?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub size_x: u32,
    pub size_y: u32,
}

impl ExperimentConfig {
    pub fn from_json_file(path: PathBuf) -> Result<ExperimentConfig> {
        let json_str = std::fs::read_to_string(path)?;
        let config: ExperimentConfig = serde_json::from_str(&json_str)?;
        Ok(config)
    }
}


#[cfg(test)]
pub mod test {
    use std::path::PathBuf;
    use crate::config::ExperimentConfig;

    #[test]
    fn test_read_from_file() {
        let path = PathBuf::from("configs/test.json"); // TODO
        let config = ExperimentConfig::from_json_file(path).unwrap();
        assert!(config.size_x == 1);
        assert!(config.size_y == 3);
    }

}

