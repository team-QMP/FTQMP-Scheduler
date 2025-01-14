pub mod test_generator;

pub use test_generator::TestGenerator;

use crate::program::Program;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum GeneratorKind {
    Test,
}

pub trait ProgramGenerator {
    /// Generate all programs and the time.
    fn generate(&self) -> Vec<(u64, Program)>;
}
