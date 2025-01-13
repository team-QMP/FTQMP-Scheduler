use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BinaryHeap};
use std::time::Instant;

use crate::config::SimulationConfig;
use crate::environment::Environment;
use crate::error::QMPError;
use crate::generator::ProgramGenerator;
use crate::job::Job;
use crate::program::Program;
use crate::scheduler::{apply_schedule, Scheduler};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub programs: Vec<Program>,
}

pub struct Simulator {
    config: SimulationConfig,
    env: Environment,
    generator: Box<dyn ProgramGenerator>,
    scheduler: Box<dyn Scheduler>,
    job_que: BTreeMap<u32, Job>,
    future_job_que: BinaryHeap<Job>,
    current_cycle: u128, // TODO: time (e.g., ns)?
    job_counter: u32,
}

impl Simulator {
    pub fn new(
        config: SimulationConfig,
        generator: Box<dyn ProgramGenerator>,
        scheduler: Box<dyn Scheduler>,
    ) -> Self {
        Self {
            env: Environment::new(config.size_x as i32, config.size_y as i32),
            config,
            generator,
            scheduler,
            current_cycle: 0,
            job_que: BTreeMap::new(),
            future_job_que: BinaryHeap::new(),
            job_counter: 0,
        }
    }

    pub fn run(&mut self) -> Result<SimulationResult> {
        for (t, program) in self.generator.generate() {
            let job_id = self.fresh_job_id();
            self.job_que
                .insert(job_id, Job::new(job_id, t, program.clone()));
            self.future_job_que.push(Job::new(job_id, t, program));
        }

        let mut result = Vec::new(); // TODO

        while !self.job_que.is_empty() || !self.future_job_que.is_empty() {
            while !self.future_job_que.is_empty()
                && self.future_job_que.peek().unwrap().added_time <= self.current_cycle
            {
                let job = self.future_job_que.pop().unwrap();
                self.scheduler.add_job(job);
            }

            // TODO: How to estimate the execution time of the scheduler?
            let start = Instant::now();
            let issued_programs = self.scheduler.run();
            let elapsed = start.elapsed().as_micros();
            let elapsed_cycles = elapsed.div_ceil(self.config.micro_sec_per_cycle.into());

            for (job_id, schedule) in issued_programs {
                let job = self
                    .job_que
                    .get(&job_id)
                    .ok_or_else(|| QMPError::invalid_job_id(job_id))?;
                let scheduled_program = apply_schedule(&job.program, &schedule);
                if !self.env.insert_program(&scheduled_program) {
                    return Err(QMPError::invalid_schedule_error(job.clone(), schedule));
                }
                result.push(scheduled_program);
                self.job_que.remove(&job_id);
            }

            self.current_cycle += elapsed_cycles;
        }

        Ok(SimulationResult { programs: result })
    }

    fn fresh_job_id(&mut self) -> u32 {
        self.job_counter += 1;
        self.job_counter - 1
    }
}
