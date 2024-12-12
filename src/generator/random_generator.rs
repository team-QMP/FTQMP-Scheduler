use crate::generator::ProgramGenerator;
use crate::ds::polycube::Polycube;

pub struct RandomGenerator;

impl ProgramGenerator for RandomGenerator {
    fn generate(&self) -> Vec<(u128, Polycube)> {
        // TODO
        Vec::new()
    }

    fn generate_one(&self) -> Polycube {
        unimplemented!()
    }
}
