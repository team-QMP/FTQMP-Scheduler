use crate::preprocess::Preprocessor;
use crate::program::{Cuboid, Program, ProgramFormat};

pub struct ConvertToCuboid;

impl Preprocessor for ConvertToCuboid {
    fn process(&self, program: Program) -> Program {
        match program.format() {
            ProgramFormat::Polycube(p) => {
                let cuboid = Cuboid::from(p);
                Program::new(ProgramFormat::Cuboid(cuboid))
            }
            ProgramFormat::Cuboid(_) => program,
        }
    }
}
