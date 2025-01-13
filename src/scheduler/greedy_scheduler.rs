use crate::config::SimulationConfig;
use crate::environment::Environment;
use crate::job::Job;
use crate::scheduler::{apply_schedule, JobID, Schedule, Scheduler};

use std::collections::VecDeque;

pub struct GreedyScheduler {
    job_list: VecDeque<Job>,
    env: Environment,
    config: SimulationConfig,
}

impl GreedyScheduler {
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            job_list: VecDeque::new(),
            env: Environment::new(config.size_x as i32, config.size_y as i32),
            config,
        }
    }
}

impl Scheduler for GreedyScheduler {
    fn add_job(&mut self, job: Job) {
        self.job_list.push_back(job);
    }

    fn run(&mut self) -> Vec<(JobID, Schedule)> {
        let mut res = Vec::new();
        while !self.job_list.is_empty() {
            let job = self.job_list.pop_front().unwrap();
            let mut dz = 0;
            'top: loop {
                for dx in 0..self.config.size_x {
                    for dy in 0..self.config.size_y {
                        for f in [0, 1] {
                            for rot in 0..3 {
                                let schedule = Schedule::new(dx as i32, dy as i32, dz, rot, f == 1);
                                let program = apply_schedule(&job.program, &schedule);
                                if self.env.issue_program(&program) {
                                    res.push((job.id, schedule));
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
