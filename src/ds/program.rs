use crate::ds::polycube::Polycube;

#[derive(Debug, Clone, PartialEq)]
pub enum ProgramFormat {
    Polycube(Polycube),
}

pub struct Program {
    format: ProgramFormat,
}

impl Program {
    pub fn new(format: ProgramFormat) -> Self {
        Program {
            format
        }
    }

    pub fn polycube(&self) -> Option<&Polycube> {
        match &self.format {
            ProgramFormat::Polycube(p) => Some(p),
            _ => None
        }
    }
}
