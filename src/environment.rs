use std::collections::HashSet;
use crate::ds::polycube::{Coordinate, Polycube};

#[derive(Debug, Clone)]
pub struct Environment {
    blocks: HashSet<Coordinate>,
    job_que: Vec<Polycube>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            blocks: HashSet::new(),
            job_que: Vec::new(),
        }
    }

    pub fn add_polycube(&mut self, p: &Polycube) -> bool {
        let is_collide = p.blocks().iter().any(|b| self.blocks.contains(b));
        if !is_collide {
            self.blocks.extend(p.blocks().clone());
        }
        !is_collide
    }

    pub fn add_job(&mut self, p: &Polycube) -> bool {
    }

    pub fn blocks(&self) -> &HashSet<Coordinate> {
        &self.blocks
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::environment::Environment;
    use crate::ds::polycube::{Coordinate, Polycube};

    #[test]
    fn test_environment_add_polycube() {
        let mut env = Environment::new();

        let p1 = Polycube::new(vec![Coordinate::new(0, 0, 0)]);
        let p2 = Polycube::new(vec![Coordinate::new(1, 1, 1)]);
        let p3 = Polycube::new(vec![Coordinate::new(0, 0, 0)]);

        assert!(env.add_polycube(&p1));
        assert!(env.add_polycube(&p2));
        assert!(!env.add_polycube(&p3));

        let blocks = HashSet::from([Coordinate::new(0, 0, 0), Coordinate::new(1, 1, 1)]);
        assert_eq!(env.blocks(), &blocks);
    }
}
