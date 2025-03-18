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
        assert_eq!(cuboid.original().clone(), None);
    }
}
