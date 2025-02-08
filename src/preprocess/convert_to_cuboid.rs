use crate::preprocess::Preprocessor;
use crate::program::{Cuboid, Polycube, Program, ProgramFormat};

pub struct ConvertToCuboid {
    num_cuboids: u32,
}

impl ConvertToCuboid {
    pub fn new(num_cuboids: u32) -> Self {
        Self { num_cuboids }
    }
}

impl Preprocessor for ConvertToCuboid {
    fn process(&self, program: Program) -> Program {
        match program.format() {
            ProgramFormat::Polycube(p) => {
                let min_z = p.min_z() as f32;
                let z_len = (p.max_z() - p.min_z() + 1) as f32;
                let mut poly_bins = vec![Polycube::new(Vec::new()); self.num_cuboids as usize];
                for block in p.blocks() {
                    let k = self.num_cuboids as f32;
                    let bin = f32::floor(k * (block.z as f32 - min_z) / z_len) as usize;
                    poly_bins[bin].add_block(block.clone());
                }
                let cuboids = poly_bins
                    .into_iter()
                    .filter(|poly| poly.size() > 0)
                    .map(|poly| Cuboid::from(&poly))
                    .collect();
                Program::new(ProgramFormat::Cuboid(cuboids))
            }
            ProgramFormat::Cuboid(_) => program,
        }
    }
}

#[cfg(test)]
mod test {
    use super::ConvertToCuboid;
    use crate::{
        preprocess::Preprocessor,
        program::{Coordinate, Polycube, Program, ProgramFormat},
    };

    #[test]
    fn test_convert_to_cuboids() {
        let num_cuboids = 4;
        let blocks = (2..15).map(|z| Coordinate::new(0, 0, z)).collect();
        let poly = Program::new(ProgramFormat::Polycube(Polycube::new(blocks)));
        let converter = ConvertToCuboid::new(num_cuboids);
        let cuboids = converter.process(poly).cuboid().unwrap().clone();
        assert!(cuboids.len() == num_cuboids as usize);
        assert!(cuboids.iter().all(|c| c.size_z() == 3 || c.size_z() == 4));
    }
}
