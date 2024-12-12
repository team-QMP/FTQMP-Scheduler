use crate::generator::ProgramGenerator;
use crate::ds::polycube::Polycube;

pub struct TestGenerator;

impl TestGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

impl ProgramGenerator for TestGenerator {
    fn generate(&self) -> Vec<(u128, Polycube)> {
        let prog_num = 5;
        let program = Polycube::from(&[
            (0, 0, 0),
            (0, 1, 0),
            (0, 2, 0),
            (0, 0, 1),
            (0, 1, 1),
            (0, 2, 1),
            (1, 0, 1),
            (1, 1, 1),
            (1, 2, 1),
            (2, 0, 1),
            (2, 1, 1),
            (2, 2, 1),
            (0, 0, 2),
            (0, 1, 2),
            (0, 2, 2),
            (0, 0, 3),
            (0, 1, 3),
            (0, 2, 3),
            (1, 0, 3),
            (1, 1, 3),
            (1, 2, 3),
            (2, 0, 3),
            (2, 1, 3),
            (2, 2, 3)
        ]);
        (0..prog_num).map(|_| (0, program.clone())).collect()
    }

    fn generate_one(&self) -> Polycube {
        unimplemented!()
    }
}
