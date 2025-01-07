pub mod polycube;

pub use polycube::{Coordinate, Polycube};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ProgramFormat {
    Polycube(Polycube),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Program {
    format: ProgramFormat,
}

impl Program {
    pub fn new(format: ProgramFormat) -> Self {
        Program { format }
    }

    pub fn polycube(&self) -> Option<&Polycube> {
        match &self.format {
            ProgramFormat::Polycube(p) => Some(p),
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
