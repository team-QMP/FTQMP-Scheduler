pub mod random_generator;
pub mod test_generator;

pub use test_generator::TestGenerator;

use crate::ds::polycube::Polycube;

pub trait ProgramGenerator {
    /// Generate all programs and the time.
    fn generate(&self) -> Vec<(u128, Polycube)>;
    /// Generate a program immediately.
    fn generate_one(&self) -> Polycube;
}
