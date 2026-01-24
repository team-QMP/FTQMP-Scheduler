use std::fs::File;
use std::io::Write;

use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Polycube {
    blocks: Vec<Coordinate>,
    #[serde(skip)]
    min_x: i32,
    #[serde(skip)]
    min_y: i32,
    #[serde(skip)]
    max_x: i32,
    #[serde(skip)]
    max_y: i32,
    #[serde(skip)]
    min_z: i32,
    #[serde(skip)]
    max_z: i32,
}

/// Returns (min_x, max_x, min_y, max_y, min_z, max_z)
fn calc_min_max_pos(coordinates: &[Coordinate]) -> (i32, i32, i32, i32, i32, i32) {
    coordinates.iter().fold(
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
    )
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

impl<'de> Deserialize<'de> for Polycube {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TempPolycube {
            blocks: Vec<Coordinate>,
        }

        let temp = TempPolycube::deserialize(deserializer)?;
        let (min_x, max_x, min_y, max_y, min_z, max_z) = calc_min_max_pos(&temp.blocks);

        Ok(Polycube {
            blocks: temp.blocks,
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z,
        })
    }
}

impl Polycube {
    pub fn new(blocks: Vec<Coordinate>) -> Self {
        let (min_x, max_x, min_y, max_y, min_z, max_z) = calc_min_max_pos(&blocks);

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

    pub fn blocks(&self) -> &[Coordinate] {
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

    pub fn pos(&self) -> Coordinate {
        Coordinate::new(self.min_x, self.min_y, self.min_z)
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
        for coordinate in &self.blocks {
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
    if pos_candidate_list.is_empty() {
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
    polycube
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cuboid {
    pos: Coordinate,
    size_x: usize,
    size_y: usize,
    size_z: usize,
    original: Option<Polycube>,
}

impl From<&Polycube> for Cuboid {
    fn from(item: &Polycube) -> Self {
        Self {
            pos: Coordinate::new(item.min_x(), item.min_y(), item.min_z()),
            size_x: (item.max_x() - item.min_x() + 1) as usize,
            size_y: (item.max_y() - item.min_y() + 1) as usize,
            size_z: (item.max_z() - item.min_z() + 1) as usize,
            //original: Some(item.clone()),
            original: None,
        }
    }
}

impl Cuboid {
    pub fn new(pos: Coordinate, size_x: usize, size_y: usize, size_z: usize) -> Self {
        Cuboid {
            pos,
            size_x,
            size_y,
            size_z,
            original: None,
        }
    }

    pub fn pos(&self) -> &Coordinate {
        &self.pos
    }

    pub fn x1(&self) -> i32 {
        self.pos.x
    }

    pub fn x2(&self) -> i32 {
        self.pos.x + (self.size_x as i32)
    }

    pub fn y1(&self) -> i32 {
        self.pos.y
    }

    pub fn y2(&self) -> i32 {
        self.pos.y + (self.size_y as i32)
    }

    pub fn z1(&self) -> i32 {
        self.pos.z
    }

    pub fn z2(&self) -> i32 {
        self.pos.z + (self.size_z as i32)
    }

    pub fn size_x(&self) -> usize {
        self.size_x
    }

    pub fn size_y(&self) -> usize {
        self.size_y
    }

    pub fn size_z(&self) -> usize {
        self.size_z
    }

    pub fn update_x1(&mut self, x: i32) {
        self.pos.x = x;
    }

    pub fn update_y1(&mut self, y: i32) {
        self.pos.y = y;
    }

    pub fn original(&self) -> &Option<Polycube> {
        &self.original
    }
}

pub fn is_overlap_polycubes(p1: &Polycube, p2: &Polycube) -> bool {
    p1.blocks()
        .iter()
        .any(|b1| p2.blocks().iter().any(|b2| b1 == b2))
}

pub fn is_overlap_cuboids(c1: &Cuboid, c2: &Cuboid) -> bool {
    let is_overlap_x = !(c1.x2() <= c2.x1() || c2.x2() <= c1.x1());
    let is_overlap_y = !(c1.y2() <= c2.y1() || c2.y2() <= c1.y1());
    let is_overlap_z = !(c1.z2() <= c2.z1() || c2.z2() <= c1.z1());
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

#[cfg(test)]
mod test {
    use super::{Coordinate, Cuboid, Polycube};

    #[test]
    fn test_polycube_new() {
        let p = Polycube::new(vec![Coordinate::new(0, 0, 0), Coordinate::new(0, 1, 0)]);
        assert_eq!(
            p.blocks(),
            &[Coordinate::new(0, 0, 0), Coordinate::new(0, 1, 0)]
        );
    }

    #[test]
    fn test_create_cuboid_from_polycube() {
        let p = Polycube::new(vec![Coordinate::new(1, 2, 3), Coordinate::new(2, 0, 1)]);
        let cuboid = Cuboid::from(&p);
        assert_eq!(cuboid.pos, Coordinate::new(1, 0, 1));
        assert_eq!(cuboid.size_x, 2);
        assert_eq!(cuboid.size_y, 3);
        assert_eq!(cuboid.size_z, 3);
        assert_eq!(cuboid.original().clone(), None);
    }
}
