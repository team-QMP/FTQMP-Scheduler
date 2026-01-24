use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Instruction {
    #[serde(flatten)]
    kind: InstructionKind,
    latency: i32,
}

impl Instruction {
    pub fn new(kind: InstructionKind, latency: i32) -> Self {
        Self { kind, latency }
    }

    pub fn kind(&self) -> &InstructionKind {
        &self.kind
    }

    pub fn latency(&self) -> i32 {
        self.latency
    }

    pub fn positions(&self) -> Vec<Location2D> {
        self.kind.positions()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "name")]
pub enum InstructionKind {
    H { pos: Location2D, aux: Location2D },
    S { pos: Location2D, aux: Location2D },
    Z { pos: Location2D },
    X { pos: Location2D },
    Meas { path: Vec<Location2D> },
    MP { path: Vec<Location2D> },
}

impl InstructionKind {
    pub fn positions(&self) -> Vec<Location2D> {
        match self {
            InstructionKind::H { pos, aux } | InstructionKind::S { pos, aux } => {
                vec![pos.clone(), aux.clone()]
            }
            InstructionKind::Z { pos } | InstructionKind::X { pos } => vec![pos.clone()],
            InstructionKind::Meas { path } | InstructionKind::MP { path } => path.clone(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Location2D(pub i32, pub i32);
