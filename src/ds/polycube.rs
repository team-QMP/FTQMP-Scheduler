use std::fs::File;
use std::io::Write;

use rand::Rng;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
    pub z: i32, // time
}

impl From<(i32, i32, i32)> for Coordinate {
    fn from(item: (i32, i32, i32)) -> Self {
        Coordinate {
            x: item.0,
            y: item.1,
            z: item.2
        }
    }
}

impl Coordinate {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            x,
            y,
            z
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Polycube {
    blocks: Vec<Coordinate>,
}

impl<const N: usize> From<&[(i32, i32, i32); N]> for Polycube {
    fn from(item: &[(i32, i32, i32); N]) -> Self {
        let mut blocks = Vec::new();
        for pos in item {
            blocks.push(Coordinate::from(pos.clone()));
        }
        Polycube::new(blocks)
    }
}

impl Polycube {
    pub fn new(blocks: Vec<Coordinate>) -> Self {
        Self {
            blocks,
        }
    }

    pub fn blocks(&self) -> &Vec<Coordinate> {
        &self.blocks
    }

    pub fn add_block(&mut self, coord: Coordinate) {
        // TODO: check
        self.blocks.push(coord);
    }
}

pub fn is_collide(_p1: &Polycube, _p2: &Polycube) -> bool {
    unimplemented!()
}


#[allow(dead_code)]
fn add_random_block(polycube: &mut Polycube) {
    let mut pos_candidate_list: Vec<Coordinate> = Vec::new();
    let shift_list: Vec<Coordinate> = vec![
        Coordinate{x: 1, y: 0, z: 0},
        Coordinate{x: -1, y: 0, z: 0},
        Coordinate{x: 0, y: 1, z: 0},
        Coordinate{x: 0, y: -1, z: 0},
        Coordinate{x: 0, y: 0, z: 1},
        Coordinate{x: 0, y: 0, z: -1},
    ];
    for pos in polycube.blocks() {
        for shift in &shift_list {
            let candidate = Coordinate::new(pos.x+shift.x, pos.y+shift.y, pos.z+shift.z);
            let is_in_candidate = pos_candidate_list.contains(&candidate);
            let is_in_poly_block = polycube.blocks().contains(&candidate);
            if !is_in_candidate && !is_in_poly_block {
                pos_candidate_list.push(candidate);
            }
        }
    }
    if pos_candidate_list.len() == 0 {
        pos_candidate_list.push(Coordinate{x:0, y:0, z:0});
    }

    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..pos_candidate_list.len());
    let chosen = pos_candidate_list[index].clone();
    polycube.add_block(chosen);
}

#[allow(dead_code)]
fn dump_polycube(polycube: &Polycube, filename: String) {
    let mut file = File::create(filename).unwrap();
    writeln!(file, "{:}", polycube.blocks().len()).unwrap();
    for pos in polycube.blocks() {
        writeln!(file, "{:} {:} {:}", pos.x, pos.y, pos.z).unwrap();
    }
}

#[allow(dead_code)]
fn create_random_polycube(num_block: i32) -> Polycube {
    let mut polycube = Polycube::new(Vec::new());
    for _ in 0..num_block {
        add_random_block(&mut polycube);
    }
    return polycube;
}

#[cfg(test)]
mod test {
    use crate::ds::polycube::{Coordinate, Polycube};

    #[test]
    fn test_polycube_new() {
        let p = Polycube::new(vec![Coordinate::new(0, 0, 0), Coordinate::new(0, 1, 0)]);
        assert_eq!(p.blocks(), &vec![Coordinate::new(0, 0, 0), Coordinate::new(0, 1, 0)]);
    }
}
