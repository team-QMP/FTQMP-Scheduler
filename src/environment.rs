use crate::program::{is_overlap, Program, ProgramFormat};

#[derive(Debug, Clone)]
pub struct Environment {
    programs: Vec<Program>,
    size_x: i32,
    size_y: i32,
}

impl Environment {
    pub fn new(size_x: i32, size_y: i32) -> Self {
        Self {
            programs: Vec::new(),
            size_x,
            size_y,
        }
    }

    pub fn insert_program(&mut self, p: &Program) -> bool {
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
        let is_overlap = self.programs.iter().any(|p2| is_overlap(p, p2));
        let can_insert = is_in_range && !is_overlap;
        if can_insert {
            self.programs.push(p.clone());
        }
        can_insert
    }

    pub fn programs(&self) -> &Vec<Program> {
        &self.programs
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

        assert!(env.insert_program(&p1));
        assert!(env.insert_program(&p2));
        assert!(!env.insert_program(&p3));

        let expected = vec![p1, p2];
        assert_eq!(env.programs(), &expected);
    }
}
