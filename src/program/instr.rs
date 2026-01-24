use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Instruction {
    latency: i32,
    #[serde(flatten)]
    kind: InstructionKind,
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

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Location2D(pub i32, pub i32);
