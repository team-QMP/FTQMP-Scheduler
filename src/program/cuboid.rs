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
        Self {
            pos: Coordinate::new(item.min_x(), item.min_y(), item.min_z()),
            size_x: (item.max_x() - item.min_x() + 1) as usize,
            size_y: (item.max_y() - item.min_y() + 1) as usize,
            size_z: (item.max_z() - item.min_z() + 1) as usize,
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
