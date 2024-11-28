pub mod random_generator;

use crate::ds::polycube::Polycube;

pub trait ProgramGenerator {
    fn generate(&self, num: u32) -> Vec<Polycube>;
}
