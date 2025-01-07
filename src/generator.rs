pub mod test_generator;

pub use test_generator::TestGenerator;

use crate::program::Program;

pub trait ProgramGenerator {
    /// Generate all programs and the time.
    fn generate(&self) -> Vec<(u128, Program)>;
    /// Generate a program immediately.
    fn generate_one(&self) -> Program;
}
