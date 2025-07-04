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
            env: Environment::new(config.clone()),
            config,
        }
    }
}

impl Scheduler for GreedyScheduler {
    fn add_job(&mut self, job: Job) {
        self.job_list.push_back(job);
    }

    fn run(&mut self, env: &Environment) -> Vec<(JobID, Schedule)> {
        let mut res = Vec::new();
        for job in self.take_jobs_by_batch_size() {
            let mut dz = env.global_pc();
            'top: loop {
                for dx in 0..self.config.size_x {
                    for dy in 0..self.config.size_y {
                        for f in [0, 1] {
                            for rot in 0..3 {
                                let schedule =
                                    Schedule::new(dx as i32, dy as i32, dz as i32, rot, f == 1);
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

impl GreedyScheduler {
    fn take_jobs_by_batch_size(&mut self) -> Vec<Job> {
        let take_len = if let Some(batch_size) = self.config.scheduler.batch_size {
            usize::min(self.job_list.len(), batch_size as usize)
        } else {
            self.job_list.len()
        };
        let mut taken_jobs = self.job_list.split_off(take_len);
        std::mem::swap(&mut taken_jobs, &mut self.job_list);
        taken_jobs.into()
    }
}
