use std::collections::HashSet;
use crate::ds::program::{Program, ProgramFormat};
use crate::ds::polycube::{Coordinate, Polycube};

#[derive(Debug, Clone)]
pub struct Environment {
    programs: Vec<Program>,
    max_x: i32,
    max_y: i32,
}

impl Environment {
    pub fn new(max_x: i32, max_y: i32) -> Self {
        Self {
            programs: Vec::new(),
            max_x,
            max_y,
        }
    }

    pub fn insert_program(&mut self, p: &Program) -> bool {
        // TODO: support other formats
        let polycube = p.polycube().unwrap();
        for block in polycube.blocks() {
            if block.x < 0 || self.max_x <= block.x || block.y < 0 || self.max_y <= block.y || block.z < 0 {
                return false;
            }
        }
        let is_collide = self.programs.iter().any(|p2| p2.check_conflict(p));
        if !is_collide {
            self.programs.push(p.clone());
        }
        !is_collide
    }

    pub fn programs(&self) -> &Vec<Program> {
        &self.programs
    }
}

#[cfg(test)]
mod test {
    use crate::environment::Environment;
    use crate::ds::program::{Program, ProgramFormat};
    use crate::ds::polycube::{Coordinate, Polycube};

    #[test]
    fn test_environment_add_polycube() {
        let mut env = Environment::new(100, 100);

        let p1 = Program::new(ProgramFormat::Polycube(Polycube::new(vec![Coordinate::new(0, 0, 0)])));
        let p2 = Program::new(ProgramFormat::Polycube(Polycube::new(vec![Coordinate::new(1, 1, 1)])));
        let p3 = Program::new(ProgramFormat::Polycube(Polycube::new(vec![Coordinate::new(0, 0, 0)])));

        assert!(env.insert_program(&p1));
        assert!(env.insert_program(&p2));
        assert!(!env.insert_program(&p3));

        let expected = vec![p1, p2];
        assert_eq!(env.programs(), &expected);
    }
}
