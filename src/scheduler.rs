pub mod greedy_scheduler;
pub mod lp_scheduler;

pub use greedy_scheduler::GreedyScheduler;
pub use lp_scheduler::LPScheduler;

use crate::environment::Environment;
use crate::job::{Job, JobID};
use crate::program::{Coordinate, Cuboid, Polycube, Program, ProgramFormat};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum SchedulerKind {
    Greedy,
    LP,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
pub fn apply_schedule_to_polycube(polycube: &Polycube, schedule: &Schedule) -> Polycube {
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
    Polycube::new(blocks)
}

pub fn apply_schedule_to_cuboid(cuboid: &Cuboid, schedule: &Schedule) -> Cuboid {
    if let Some(polycube) = cuboid.original() {
        let scheduled_poly = apply_schedule_to_polycube(polycube, schedule);
        Cuboid::from(&scheduled_poly)
    } else {
        let (size_x, size_y) = if schedule.rotate % 2 == 1 {
            (cuboid.size_y(), cuboid.size_x())
        } else {
            (cuboid.size_x(), cuboid.size_y())
        };
        let pos = cuboid.pos().clone() + Coordinate::new(schedule.x, schedule.y, schedule.z);
        Cuboid::new(pos, size_x, size_y, cuboid.size_z())
    }
}

pub fn apply_schedule(program: &Program, schedule: &Schedule) -> Program {
    assert!(0 <= schedule.rotate && schedule.rotate < 3);
    match program.format() {
        ProgramFormat::Polycube(polycube) => {
            let scheduled = apply_schedule_to_polycube(polycube, schedule);
            Program::new(ProgramFormat::Polycube(scheduled))
        }
        ProgramFormat::Cuboid(cuboids) => {
            let cuboids = cuboids
                .iter()
                .map(|c| apply_schedule_to_cuboid(c, schedule))
                .collect();
            Program::new(ProgramFormat::Cuboid(cuboids))
        }
    }
}

pub trait Scheduler {
    fn add_job(&mut self, job: Job);
    fn run(&mut self, env: &Environment) -> Vec<(JobID, Schedule)>;
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
