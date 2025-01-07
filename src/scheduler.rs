pub mod greedy_scheduler;
pub mod lp_scheduler;

pub use greedy_scheduler::GreedyScheduler;
pub use lp_scheduler::LPScheduler;

use crate::job::{Job, JobID};
use crate::program::{Coordinate, Polycube, Program, ProgramFormat};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Schedule {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub rotate: i32, // 0 <= rotate < 3
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

/// Note: flip -> rotate -> (adjust coordinates) -> shift
/// Support polycube format only.
pub fn apply_schedule(program: &Program, schedule: &Schedule) -> Program {
    assert!(0 <= schedule.rotate && schedule.rotate < 3);
    let polycube = program.polycube().unwrap(); // TODO: error handling
    let mut blocks = Vec::new();
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    for block in polycube.blocks() {
        let (x, y) = if schedule.flip {
            (-block.x, block.y)
        } else {
            (block.x, block.y)
        };
        let (x, y) = match schedule.rotate {
            0 => (x, y),
            1 => (-y, x),
            2 => (-x, y),
            _ => (y, -x),
        };
        min_x = i32::min(min_x, x);
        min_y = i32::min(min_y, y);
        blocks.push(Coordinate::new(x, y, block.z));
    }

    let blocks = blocks
        .into_iter()
        .map(|coord| {
            Coordinate::new(
                coord.x - min_x + schedule.x,
                coord.y - min_y + schedule.y,
                coord.z + schedule.z,
            )
        })
        .collect();
    Program::new(ProgramFormat::Polycube(Polycube::new(blocks)))
}

pub trait Scheduler {
    fn add_job(&mut self, job: Job);
    fn run(&mut self) -> Vec<(JobID, Schedule)>;
}

#[cfg(test)]
mod test {
    use crate::program::{Coordinate, Polycube, Program, ProgramFormat};
    use crate::scheduler::{apply_schedule, Schedule};

    #[test]
    fn test_apply_schedule() {
        let p = Program::new(ProgramFormat::Polycube(Polycube::new(vec![
            Coordinate::new(2, 1, 0),
            Coordinate::new(1, 2, 0),
        ])));
        let s = Schedule::new(1, 10, 3, 1, true);
        let scheduled = apply_schedule(&p, &s);
        let actual = scheduled.polycube().unwrap();
        let expected = Polycube::new(vec![Coordinate::new(2, 10, 3), Coordinate::new(1, 11, 3)]);
        assert_eq!(*actual, expected);
    }
}
