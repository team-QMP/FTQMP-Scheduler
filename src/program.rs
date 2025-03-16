pub mod cuboid;
pub mod polycube;

pub use cuboid::Cuboid;
pub use polycube::{Coordinate, Polycube};

use serde::{Deserialize, Serialize};

pub type ProgramCounter = u64;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ProgramFormat {
    Polycube(Polycube),
    Cuboid(Vec<Cuboid>),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Program {
    #[serde(flatten)]
    format: ProgramFormat,
}

impl Program {
    pub fn new(format: ProgramFormat) -> Self {
        Program { format }
    }

    pub fn is_polycube(&self) -> bool {
        matches!(&self.format, ProgramFormat::Polycube(_))
    }

    pub fn polycube(&self) -> Option<&Polycube> {
        match &self.format {
            ProgramFormat::Polycube(p) => Some(p),
            _ => None,
        }
    }

    pub fn is_cuboid(&self) -> bool {
        matches!(&self.format, ProgramFormat::Cuboid(_))
    }

    pub fn cuboid(&self) -> Option<&Vec<Cuboid>> {
        match &self.format {
            ProgramFormat::Cuboid(c) => Some(c),
            _ => None,
        }
    }

    pub fn format(&self) -> &ProgramFormat {
        &self.format
    }

    pub fn max_z(&self) -> i32 {
        match self.format() {
            ProgramFormat::Polycube(p) => p.max_z(),
            ProgramFormat::Cuboid(cs) => cs.iter().map(|c| c.z2() - 1).max().unwrap(),
        }
    }

    /// Returns the burst time (= execution time) in cycles
    pub fn burst_time(&self) -> u64 {
        match &self.format {
            ProgramFormat::Polycube(poly) => {
                assert!(poly.max_z() >= poly.min_z());
                (poly.max_z() - poly.min_z() + 1) as u64
            }
            ProgramFormat::Cuboid(cs) => {
                let (min_z, max_z) = cs.iter().fold((u64::MAX, u64::MIN), |(min_z, max_z), c| {
                    let z_pos = c.pos().z as u64;
                    (
                        u64::min(min_z, z_pos),
                        u64::max(max_z, z_pos + c.size_z() as u64),
                    )
                });
                max_z - min_z
            }
        }
    }
}

pub fn is_overlap_polycubes(p1: &Polycube, p2: &Polycube) -> bool {
    p1.blocks()
        .iter()
        .any(|b1| p2.blocks().iter().any(|b2| b1 == b2))
}

pub fn is_overlap_cuboids(c1: &Cuboid, c2: &Cuboid) -> bool {
    let is_overlap_x = !(c1.pos().x + c1.size_x() as i32 <= c2.pos().x
        || c2.pos().x + c2.size_x() as i32 <= c1.pos().x);
    let is_overlap_y = !(c1.pos().y + c1.size_y() as i32 <= c2.pos().y
        || c2.pos().y + c2.size_y() as i32 <= c1.pos().y);
    let is_overlap_z = !(c1.pos().z + c1.size_z() as i32 <= c2.pos().z
        || c2.pos().z + c2.size_z() as i32 <= c1.pos().z);
    is_overlap_x && is_overlap_y && is_overlap_z
}

pub fn is_overlap_polycube_cuboid(p: &Polycube, c: &Cuboid) -> bool {
    p.blocks().iter().any(|b| {
        let cp = c.pos();
        cp.x <= b.x
            && b.x < cp.x + c.size_x() as i32
            && cp.y <= b.y
            && b.y <= cp.y + c.size_y() as i32
            && cp.z <= b.z
            && b.z <= cp.z + c.size_z() as i32
    })
}

pub fn is_overlap(p1: &Program, p2: &Program) -> bool {
    match (p1.format(), p2.format()) {
        (ProgramFormat::Polycube(p1), ProgramFormat::Polycube(p2)) => is_overlap_polycubes(p1, p2),
        (ProgramFormat::Polycube(p), ProgramFormat::Cuboid(cs))
        | (ProgramFormat::Cuboid(cs), ProgramFormat::Polycube(p)) => {
            cs.iter().any(|c| is_overlap_polycube_cuboid(p, c))
        }
        (ProgramFormat::Cuboid(cs1), ProgramFormat::Cuboid(cs2)) => cs1
            .iter()
            .any(|c1| cs2.iter().any(|c2| is_overlap_cuboids(c1, c2))),
    }
}

/// Divide a program into two parts [0, z) and [z, inf].
pub fn cut_program_at_z(p: Program, z: i32) -> (Option<Program>, Option<Program>) {
    // TODO: remove .clone
    match p.format() {
        ProgramFormat::Polycube(_poly) => {
            unimplemented!() // currently unsupported
        }
        ProgramFormat::Cuboid(cs) => {
            let mut below = Vec::new();
            let mut above = Vec::new();
            for c in cs {
                if c.pos().z + (c.size_z() as i32) < z {
                    below.push(c.clone());
                } else if z <= c.pos().z {
                    above.push(c.clone());
                } else {
                    let below_c = Cuboid::new(
                        c.pos().clone(),
                        c.size_x(),
                        c.size_y(),
                        (z - c.pos().z) as usize,
                    );
                    let above_c = Cuboid::new(
                        Coordinate::new(c.pos().x, c.pos().y, z),
                        c.size_x(),
                        c.size_y(),
                        c.size_z() - below_c.size_z(),
                    );
                    below.push(below_c);
                    above.push(above_c);
                }
            }
            let below = if below.is_empty() {
                None
            } else {
                let format = ProgramFormat::Cuboid(below);
                Some(Program::new(format))
            };
            let above = if above.is_empty() {
                None
            } else {
                let format = ProgramFormat::Cuboid(above);
                Some(Program::new(format))
            };
            (below, above)
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::program::{cut_program_at_z, Coordinate, Cuboid, Program, ProgramFormat};

    #[test]
    fn test_cut_program_at_z() {
        let c1 = Cuboid::new(Coordinate::new(0, 0, 0), 1, 1, 2);
        let c2 = Cuboid::new(Coordinate::new(0, 0, 2), 1, 1, 3);
        let c3 = Cuboid::new(Coordinate::new(0, 0, 5), 1, 1, 4);
        let program = Program::new(ProgramFormat::Cuboid(vec![c1.clone(), c2, c3.clone()]));
        let (below, above) = cut_program_at_z(program, 3);
        let below_cs = below.unwrap().cuboid().unwrap().clone();
        let above_cs = above.unwrap().cuboid().unwrap().clone();
        let c2_below = Cuboid::new(Coordinate::new(0, 0, 2), 1, 1, 1);
        let c2_above = Cuboid::new(Coordinate::new(0, 0, 3), 1, 1, 2);
        assert_eq!(below_cs, vec![c1, c2_below]);
        assert_eq!(above_cs, vec![c2_above, c3]);
    }
}
