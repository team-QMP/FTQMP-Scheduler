use crate::scheduler::{JobID, Scheduler, Schedule, apply_schedule};
use crate::ds::program::Program;
use crate::ds::polycube::Polycube;
use crate::environment::Environment;
use crate::config::SimulationConfig;

use std::collections::VecDeque;

pub struct GreedyScheduler {
    program_list: VecDeque<(JobID, Program)>,
    env: Environment,
    config: SimulationConfig,
}

impl GreedyScheduler {
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            program_list: VecDeque::new(),
            env: Environment::new(config.size_x as i32, config.size_y as i32),
            config,
        }
    }
}

impl Scheduler for GreedyScheduler {
    fn add_job(&mut self, job_id: JobID, program: Program) {
        program.polycube().unwrap();
        self.program_list.push_back((job_id, program));
    }

    fn run(&mut self) -> Vec<(JobID, Schedule)> {
        let mut res = Vec::new();
        while self.program_list.len() != 0 {
            let (job_id, program) = self.program_list.pop_front().unwrap();
            let mut dz = 0;
            'top: loop {
                for dx in 0..self.config.size_x {
                    for dy in 0..self.config.size_y {
                        for f in [0, 1] {
                            for rot in 0..3 {
                                let schedule = Schedule::new(dx as i32, dy as i32, dz as i32, rot, f == 1);
                                let program = apply_schedule(&program, &schedule);
                                if self.env.insert_program(&program) {
                                    res.push((job_id, schedule));
                                    break 'top;
                                }
                            }
                        }
                    }
                }
                dz += 1;
            }
        }

        res
    }
}
