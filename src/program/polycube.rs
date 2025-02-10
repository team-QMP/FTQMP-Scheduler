use std::fs::File;
use std::io::Write;

use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_tuple::{Deserialize_tuple, Serialize_tuple};

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize_tuple, Serialize_tuple,
)]
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
            z: item.2,
        }
    }
}

impl std::ops::Add for Coordinate {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Coordinate {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Polycube {
    blocks: Vec<Coordinate>,
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
    min_z: i32,
    max_z: i32,
}

impl<const N: usize> From<&[(i32, i32, i32); N]> for Polycube {
    fn from(item: &[(i32, i32, i32); N]) -> Self {
        let mut blocks = Vec::new();
        for pos in item {
            blocks.push(Coordinate::from(*pos));
        }
        Polycube::new(blocks)
    }
}

impl Polycube {
    pub fn new(blocks: Vec<Coordinate>) -> Self {
        let (min_x, max_x, min_y, max_y, min_z, max_z) = blocks.iter().fold(
            (i32::MAX, i32::MIN, i32::MAX, i32::MIN, i32::MAX, i32::MIN),
            |(min_x, max_x, min_y, max_y, min_z, max_z), pos| {
                (
                    i32::min(min_x, pos.x),
                    i32::max(max_x, pos.x),
                    i32::min(min_y, pos.y),
                    i32::max(max_y, pos.y),
                    i32::min(min_z, pos.z),
                    i32::max(max_z, pos.z),
                )
            },
        );
        Self {
            blocks,
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z,
        }
    }

    pub fn blocks(&self) -> &Vec<Coordinate> {
        &self.blocks
    }

    pub fn min_x(&self) -> i32 {
        self.min_x
    }

    pub fn max_x(&self) -> i32 {
        self.max_x
    }

    pub fn min_y(&self) -> i32 {
        self.min_y
    }

    pub fn max_y(&self) -> i32 {
        self.max_y
    }

    pub fn min_z(&self) -> i32 {
        self.min_z
    }

    pub fn max_z(&self) -> i32 {
        self.max_z
    }

    pub fn add_block(&mut self, coord: Coordinate) {
        self.min_x = i32::min(self.min_x, coord.x);
        self.min_y = i32::min(self.min_y, coord.y);
        self.min_z = i32::min(self.min_z, coord.z);
        self.max_x = i32::max(self.max_x, coord.x);
        self.max_y = i32::max(self.max_y, coord.y);
        self.max_z = i32::max(self.max_z, coord.z);
        self.blocks.push(coord);
    }

    pub fn size(&self) -> i32 {
        self.blocks.len() as i32
    }

    pub fn index_to_xyz(&self, index: i32) -> Coordinate {
        self.blocks[index as usize].clone()
    }

    pub fn print(&self) {
        for i in 0..self.size() {
            let coordinate = self.index_to_xyz(i);
            println!("({}, {}, {})", coordinate.x, coordinate.y, coordinate.z);
        }
    }
}

pub fn is_collide(_p1: &Polycube, _p2: &Polycube) -> bool {
    unimplemented!()
}

#[allow(dead_code)]
fn add_random_block(polycube: &mut Polycube) {
    let mut pos_candidate_list: Vec<Coordinate> = Vec::new();
    let shift_list: Vec<Coordinate> = vec![
        Coordinate { x: 1, y: 0, z: 0 },
        Coordinate { x: -1, y: 0, z: 0 },
        Coordinate { x: 0, y: 1, z: 0 },
        Coordinate { x: 0, y: -1, z: 0 },
        Coordinate { x: 0, y: 0, z: 1 },
        Coordinate { x: 0, y: 0, z: -1 },
    ];
    for pos in polycube.blocks() {
        for shift in &shift_list {
            let candidate = Coordinate::new(pos.x + shift.x, pos.y + shift.y, pos.z + shift.z);
            let is_in_candidate = pos_candidate_list.contains(&candidate);
            let is_in_poly_block = polycube.blocks().contains(&candidate);
            if !is_in_candidate && !is_in_poly_block {
                pos_candidate_list.push(candidate);
            }
        }
    }
    if pos_candidate_list.len() == 0 {
        pos_candidate_list.push(Coordinate { x: 0, y: 0, z: 0 });
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
pub fn create_random_polycube(num_block: i32) -> Polycube {
    let mut polycube = Polycube::new(Vec::new());
    for _ in 0..num_block {
        add_random_block(&mut polycube);
    }
    return polycube;
}

#[cfg(test)]
mod test {
    use crate::program::{Coordinate, Polycube};

    #[test]
    fn test_polycube_new() {
        let p = Polycube::new(vec![Coordinate::new(0, 0, 0), Coordinate::new(0, 1, 0)]);
        assert_eq!(
            p.blocks(),
            &vec![Coordinate::new(0, 0, 0), Coordinate::new(0, 1, 0)]
        );
    }
}
