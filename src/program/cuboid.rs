use serde::{Deserialize, Serialize};

use crate::program::{Coordinate, Polycube};

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
        let (min_x, max_x, min_y, max_y, min_z, max_z) = item.blocks().iter().fold(
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
            pos: Coordinate::new(min_x, min_y, min_z),
            size_x: (max_x - min_x + 1) as usize,
            size_y: (max_y - min_y + 1) as usize,
            size_z: (max_z - min_z + 1) as usize,
            original: Some(item.clone()),
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
    pub fn size_x(&self) -> usize {
        self.size_x
    }
    pub fn size_y(&self) -> usize {
        self.size_y
    }
    pub fn size_z(&self) -> usize {
        self.size_z
    }
    pub fn original(&self) -> &Option<Polycube> {
        &self.original
    }
}

#[cfg(test)]
mod test {
    use crate::program::{Coordinate, Cuboid, Polycube};

    #[test]
    fn test_create_cuboid_from_polycube() {
        let p = Polycube::new(vec![Coordinate::new(1, 2, 3), Coordinate::new(2, 0, 1)]);
        let cuboid = Cuboid::from(&p);
        assert_eq!(cuboid.pos, Coordinate::new(1, 0, 1));
        assert_eq!(cuboid.size_x, 2);
        assert_eq!(cuboid.size_y, 3);
        assert_eq!(cuboid.size_z, 3);
        assert_eq!(cuboid.original().clone().unwrap(), p);
    }
}
