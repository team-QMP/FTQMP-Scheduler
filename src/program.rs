pub mod instr;
pub mod shape;

pub use shape::{Coordinate, Cuboid, Polycube};

use instr::Instruction;
use shape::{is_overlap_cuboids, is_overlap_polycube_cuboid, is_overlap_polycubes};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub type ProgramCounter = u64;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum InternalFormat {
    Polycube(Polycube),
    Cuboid(Vec<Cuboid>),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Program {
    #[serde(default, rename = "instructions", alias = "instrs")]
    instrs: Vec<Instruction>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    #[serde(skip)]
    internal_format: Option<InternalFormat>,
}

impl Program {
    pub fn new(format: InternalFormat) -> Self {
        Program {
            instrs: Vec::new(),
            name: None,
            internal_format: Some(format),
        }
    }

    pub fn from_json_file(path: PathBuf) -> Result<Self> {
        let json_str = std::fs::read_to_string(path)?;
        let mut program: Program = serde_json::from_str(&json_str)?;
        program.build_internal_from_instructions()?;
        Ok(program)
    }

    pub fn with_instructions(instrs: Vec<Instruction>) -> Self {
        Program {
            instrs,
            name: None,
            internal_format: None,
        }
    }

    pub fn build_internal_from_instructions(&mut self) -> Result<()> {
        if self.has_format() {
            return Ok(());
        }
        if self.instrs.is_empty() {
            anyhow::bail!("program has no instructions to build internal representation");
        }

        let mut blocks = Vec::new();
        let mut current_z = 0;
        for instr in &self.instrs {
            let latency = instr.latency();
            if latency <= 0 {
                anyhow::bail!("instruction latency must be positive: {latency}");
            }
            let positions = instr.positions();
            for dz in 0..latency {
                let z = current_z + dz;
                for pos in &positions {
                    blocks.push(Coordinate::new(pos.0, pos.1, z));
                }
            }
            current_z += latency;
        }

        if blocks.is_empty() {
            anyhow::bail!("program has no occupied positions to build internal representation");
        }

        let polycube = Polycube::new(blocks);
        self.internal_format = Some(InternalFormat::Cuboid(vec![Cuboid::from(&polycube)]));
        Ok(())
    }

    pub fn set_format(&mut self, format: InternalFormat) {
        self.internal_format = Some(format);
    }

    pub fn has_format(&self) -> bool {
        self.internal_format.is_some()
    }

    pub fn format_opt(&self) -> Option<&InternalFormat> {
        self.internal_format.as_ref()
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instrs
    }

    pub fn is_polycube(&self) -> bool {
        matches!(&self.internal_format, Some(InternalFormat::Polycube(_)))
    }

    pub fn polycube(&self) -> Option<&Polycube> {
        match &self.internal_format {
            Some(InternalFormat::Polycube(p)) => Some(p),
            _ => None,
        }
    }

    pub fn is_cuboid(&self) -> bool {
        matches!(&self.internal_format, Some(InternalFormat::Cuboid(_)))
    }

    pub fn cuboid(&self) -> Option<&Vec<Cuboid>> {
        match &self.internal_format {
            Some(InternalFormat::Cuboid(c)) => Some(c),
            _ => None,
        }
    }

    pub fn format(&self) -> &InternalFormat {
        self.internal_format
            .as_ref()
            .expect("internal format must be set before use")
    }

    pub fn pos(&self) -> Coordinate {
        match self.format() {
            InternalFormat::Polycube(p) => p.pos(),
            InternalFormat::Cuboid(cs) => {
                assert!(cs.len() == 1);
                cs[0].pos().clone()
            }
        }
    }

    pub fn z2(&self) -> i32 {
        match self.format() {
            InternalFormat::Polycube(p) => p.max_z() + 1,
            InternalFormat::Cuboid(cs) => cs.iter().map(|c| c.z2()).max().unwrap(),
        }
    }

    /// Returns the burst time (= execution time) in cycles
    pub fn burst_time(&self) -> u64 {
        match self.format() {
            InternalFormat::Polycube(poly) => {
                assert!(poly.max_z() >= poly.min_z());
                (poly.max_z() - poly.min_z() + 1) as u64
            }
            InternalFormat::Cuboid(cs) => {
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

pub fn is_overlap(p1: &Program, p2: &Program) -> bool {
    match (p1.format(), p2.format()) {
        (InternalFormat::Polycube(p1), InternalFormat::Polycube(p2)) => {
            is_overlap_polycubes(p1, p2)
        }
        (InternalFormat::Polycube(p), InternalFormat::Cuboid(cs))
        | (InternalFormat::Cuboid(cs), InternalFormat::Polycube(p)) => {
            cs.iter().any(|c| is_overlap_polycube_cuboid(p, c))
        }
        (InternalFormat::Cuboid(cs1), InternalFormat::Cuboid(cs2)) => cs1
            .iter()
            .any(|c1| cs2.iter().any(|c2| is_overlap_cuboids(c1, c2))),
    }
}

/// Divide a program into two parts [0, z) and [z, inf].
pub fn cut_program_at_z(p: Program, z: i32) -> (Option<Program>, Option<Program>) {
    // TODO: remove .clone
    match p.format() {
        InternalFormat::Polycube(_poly) => {
            unimplemented!() // currently unsupported
        }
        InternalFormat::Cuboid(cs) => {
            let mut below = Vec::new();
            let mut above = Vec::new();
            for c in cs {
                if c.z2() <= z {
                    below.push(c.clone());
                } else if z <= c.z1() {
                    above.push(c.clone());
                } else {
                    let below_c = Cuboid::new(
                        c.pos().clone(),
                        c.size_x(),
                        c.size_y(),
                        (z - c.z1()) as usize,
                    );
                    let above_c = Cuboid::new(
                        Coordinate::new(c.x1(), c.y1(), z),
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
                let format = InternalFormat::Cuboid(below);
                Some(Program::new(format))
            };
            let above = if above.is_empty() {
                None
            } else {
                let format = InternalFormat::Cuboid(above);
                Some(Program::new(format))
            };
            (below, above)
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::program::{
        cut_program_at_z, instr::Instruction, instr::InstructionKind, instr::Location2D,
        Coordinate, Cuboid, InternalFormat, Program,
    };
    use std::path::PathBuf;

    #[test]
    fn test_cut_program_at_z() {
        let c1 = Cuboid::new(Coordinate::new(0, 0, 0), 1, 1, 2);
        let c2 = Cuboid::new(Coordinate::new(0, 0, 2), 1, 1, 3);
        let c3 = Cuboid::new(Coordinate::new(0, 0, 5), 1, 1, 4);
        let program = Program::new(InternalFormat::Cuboid(vec![c1.clone(), c2, c3.clone()]));
        let (below, above) = cut_program_at_z(program, 3);
        let below_cs = below.unwrap().cuboid().unwrap().clone();
        let above_cs = above.unwrap().cuboid().unwrap().clone();
        let c2_below = Cuboid::new(Coordinate::new(0, 0, 2), 1, 1, 1);
        let c2_above = Cuboid::new(Coordinate::new(0, 0, 3), 1, 1, 2);
        assert_eq!(below_cs, vec![c1, c2_below]);
        assert_eq!(above_cs, vec![c2_above, c3]);
    }

    #[test]
    fn test_parse_program_json() {
        let program = Program::from_json_file(PathBuf::from("samples/program.json"))
            .expect("parse samples/program.json");
        let instrs = program.instructions();
        assert_eq!(instrs.len(), 6);

        assert_eq!(instrs[0].latency(), 2);
        match instrs[0].kind() {
            InstructionKind::H { pos, aux } => {
                assert_eq!(*pos, Location2D(0, 0));
                assert_eq!(*aux, Location2D(1, 0));
            }
            other => panic!("expected H, got {other:?}"),
        }

        assert_eq!(instrs[1].latency(), 1);
        match instrs[1].kind() {
            InstructionKind::S { pos, aux } => {
                assert_eq!(*pos, Location2D(1, 0));
                assert_eq!(*aux, Location2D(2, 0));
            }
            other => panic!("expected S, got {other:?}"),
        }

        assert_eq!(instrs[2].latency(), 3);
        match instrs[2].kind() {
            InstructionKind::Z { pos } => assert_eq!(*pos, Location2D(2, 1)),
            other => panic!("expected Z, got {other:?}"),
        }

        assert_eq!(instrs[3].latency(), 3);
        match instrs[3].kind() {
            InstructionKind::X { pos } => assert_eq!(*pos, Location2D(2, 2)),
            other => panic!("expected X, got {other:?}"),
        }

        assert_eq!(instrs[4].latency(), 4);
        match instrs[4].kind() {
            InstructionKind::Meas { path } => {
                assert_eq!(
                    path,
                    &vec![Location2D(0, 0), Location2D(1, 0), Location2D(1, 1)]
                );
            }
            other => panic!("expected Meas, got {other:?}"),
        }

        assert_eq!(instrs[5].latency(), 4);
        match instrs[5].kind() {
            InstructionKind::MP { path } => {
                assert_eq!(path, &vec![Location2D(2, 0), Location2D(2, 1)]);
            }
            other => panic!("expected MP, got {other:?}"),
        }
    }

    #[test]
    fn test_build_internal_from_instructions() {
        let instrs = vec![
            Instruction::new(
                InstructionKind::H {
                    pos: Location2D(0, 0),
                    aux: Location2D(0, 1),
                },
                2,
            ),
            Instruction::new(
                InstructionKind::Z {
                    pos: Location2D(1, 0),
                },
                1,
            ),
        ];
        let mut program = Program::with_instructions(instrs);
        program
            .build_internal_from_instructions()
            .expect("build internal format");

        let cuboids = program.cuboid().expect("has cuboid");
        assert_eq!(cuboids.len(), 1);
        assert_eq!(cuboids[0].pos(), &Coordinate::new(0, 0, 0));
        assert_eq!(cuboids[0].size_x(), 2);
        assert_eq!(cuboids[0].size_y(), 2);
        assert_eq!(cuboids[0].size_z(), 3);
    }
}
