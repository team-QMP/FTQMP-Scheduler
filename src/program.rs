pub mod cuboid;
pub mod polycube;

pub use cuboid::Cuboid;
pub use polycube::{Coordinate, Polycube};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ProgramFormat {
    Polycube(Polycube),
    Cuboid(Cuboid),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Program {
    format: ProgramFormat,
}

impl Program {
    pub fn new(format: ProgramFormat) -> Self {
        Program { format }
    }

    pub fn is_polycube(&self) -> bool {
        match &self.format {
            ProgramFormat::Polycube(_) => true,
            _ => false,
        }
    }
    pub fn polycube(&self) -> Option<&Polycube> {
        match &self.format {
            ProgramFormat::Polycube(p) => Some(p),
            _ => None,
        }
    }

    pub fn is_cuboid(&self) -> bool {
        match &self.format {
            ProgramFormat::Cuboid(_) => true,
            _ => false,
        }
    }
    pub fn cuboid(&self) -> Option<&Cuboid> {
        match &self.format {
            ProgramFormat::Cuboid(c) => Some(c),
            _ => None,
        }
    }

    pub fn format(&self) -> &ProgramFormat {
        &self.format
    }

    pub fn check_conflict(&self, other: &Program) -> bool {
        match (self.polycube(), other.polycube()) {
            (Some(p1), Some(p2)) => p1
                .blocks()
                .iter()
                .any(|b1| p2.blocks().iter().any(|b2| b1 == b2)),
            _ => unimplemented!(),
        }
    }
}
