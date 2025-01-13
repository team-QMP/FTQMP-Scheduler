use crate::program::{is_overlap, Program, ProgramFormat};

#[derive(Debug, Clone)]
pub struct Environment {
    issued_programs: Vec<Program>,
    size_x: i32,
    size_y: i32,
    current_cycle: u128,
}

impl Environment {
    pub fn new(size_x: i32, size_y: i32) -> Self {
        Self {
            issued_programs: Vec::new(),
            size_x,
            size_y,
            current_cycle: 0,
        }
    }

    pub fn issue_program(&mut self, p: &Program) -> bool {
        let is_in_range = match p.format() {
            ProgramFormat::Polycube(polycube) => polycube.blocks().iter().all(|b| {
                0 <= b.x && b.x < self.size_x && 0 <= b.y && b.y < self.size_y && 0 <= b.z
            }),
            ProgramFormat::Cuboid(c) => {
                let pos = c.pos();
                0 <= pos.x
                    && pos.x + (c.size_x() as i32) <= self.size_x
                    && 0 <= pos.y
                    && pos.y + (c.size_y() as i32) <= self.size_y
                    && 0 <= pos.z
            }
        };
        let is_overlap = self.issued_programs.iter().any(|p2| is_overlap(p, p2));
        let can_insert = is_in_range && !is_overlap;
        if can_insert {
            self.issued_programs.push(p.clone());
        }
        can_insert
    }

    pub fn issued_programs(&self) -> &Vec<Program> {
        &self.issued_programs
    }

    pub fn current_cycle(&self) -> u128 {
        self.current_cycle
    }

    pub fn incr_cycle(&mut self, cycle: u128) {
        self.current_cycle += cycle;
    }
}

#[cfg(test)]
mod test {
    use crate::environment::Environment;
    use crate::program::{Coordinate, Polycube, Program, ProgramFormat};

    #[test]
    fn test_environment_add_polycube() {
        let mut env = Environment::new(100, 100);

        let p1 = Program::new(ProgramFormat::Polycube(Polycube::new(vec![
            Coordinate::new(0, 0, 0),
        ])));
        let p2 = Program::new(ProgramFormat::Polycube(Polycube::new(vec![
            Coordinate::new(1, 1, 1),
        ])));
        let p3 = Program::new(ProgramFormat::Polycube(Polycube::new(vec![
            Coordinate::new(0, 0, 0),
        ])));

        assert!(env.issue_program(&p1));
        assert!(env.issue_program(&p2));
        assert!(!env.issue_program(&p3));

        let expected = vec![p1, p2];
        assert_eq!(env.issued_programs(), &expected);
    }
}
