#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Schedule {
    pub x: i32,
    pub y: i32,
    pub z: i32, // time
    pub rotate: i32,
    pub flip: bool,
}
impl Schedule {
    pub fn new(x: i32, y: i32, z: i32, rotate: i32, flip: bool) -> Self {
        Self {
            x,
            y,
            z,
            rotate,
            flip,
        }
    }
}
