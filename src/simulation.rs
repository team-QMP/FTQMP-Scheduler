use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BinaryHeap};
use std::time::Instant;

use crate::config::SimulationConfig;
use crate::environment::Environment;
use crate::error::QMPError;
use crate::generator::ProgramGenerator;
use crate::job::Job;
use crate::preprocess::{ConvertToCuboid, PreprocessKind, Preprocessor};
use crate::program::Program;
use crate::scheduler::{apply_schedule, Scheduler};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub programs: Vec<Program>,
    pub delays: Vec<(u64, u64)>,
}

pub struct Simulator {
    config: SimulationConfig,
    env: Environment,
    generator: Box<dyn ProgramGenerator>,
    scheduler: Box<dyn Scheduler>,
    job_que: BinaryHeap<Job>,
    /// The requested but not scheduled job list
    waiting_jobs: BTreeMap<u32, Job>,
    job_counter: u32,
    /// The number of cycles elapsed since the start of the simulation.
    current_cycle: u64,
    /// A delay (s, d) means a delay of $d$ cycles occured in the $s$-th step.
    delays: Vec<(u64, u64)>,
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
            job_que: BinaryHeap::new(),
            waiting_jobs: BTreeMap::new(),
            job_counter: 0,
            current_cycle: 0,
            delays: Vec::new(),
        }
    }

    pub fn run(mut self) -> Result<SimulationResult> {
        let preprocessors: Vec<_> = self
            .config
            .preprocessor
            .processes
            .iter()
            .map(|kind| match kind {
                PreprocessKind::ConvertToCuboid => ConvertToCuboid {},
            })
            .collect();
        for (t, program) in self.generator.generate() {
            let job_id = self.fresh_job_id();
            let program = preprocessors.iter().fold(program, |p, pp| pp.process(p));
            self.job_que.push(Job::new(job_id, t, program));
        }

        let mut result = Vec::new();

        while !self.job_que.is_empty() || !self.waiting_jobs.is_empty() {
            // When the scheduler does not have jobs to be scheduled and there is a job in the job queue,
            // then change the current cycle to the time when the new job is requested.
            if self.waiting_jobs.is_empty()
                && !self.job_que.is_empty()
                && self.job_que.peek().unwrap().requested_time > self.current_cycle
            {
                let forward_cycles =
                    self.job_que.peek().unwrap().requested_time - self.current_cycle;
                self.current_cycle += forward_cycles;
                self.env.incr_pc(forward_cycles);
            }

            while !self.job_que.is_empty()
                && self.job_que.peek().unwrap().requested_time <= self.current_cycle
            {
                let job = self.job_que.pop().unwrap();
                self.waiting_jobs.insert(job.id, job.clone());
                self.scheduler.add_job(job);
            }

            // TODO: How to estimate the execution time of the scheduler?
            let start = Instant::now();
            let issued_programs = self.scheduler.run(&self.env);
            let elapsed_cycles = start
                .elapsed()
                .as_micros()
                .div_ceil(self.config.micro_sec_per_cycle.into())
                as u64;

            let mut scheduled_point = u64::MAX;
            for (job_id, schedule) in issued_programs {
                if (schedule.z as u64) < self.env.program_counter() {
                    return Err(QMPError::ViolateTimingConstraint.into());
                }
                scheduled_point = u64::min(scheduled_point, schedule.z as u64);
                let job = self
                    .waiting_jobs
                    .get(&job_id)
                    .ok_or_else(|| QMPError::invalid_job_id(job_id))?;
                let scheduled_program = apply_schedule(&job.program, &schedule);
                if !self.env.issue_program(&scheduled_program) {
                    return Err(QMPError::invalid_schedule_error(job.clone(), schedule));
                }
                result.push(scheduled_program);
                self.waiting_jobs.remove(&job_id);
            }

            self.current_cycle += elapsed_cycles;
            // When the program point specified by the scheduler (= minimum z position of schedules) is reached, program execution is stopped until the result is returned.
            let count_to_scheduled_point = scheduled_point - self.env.program_counter();
            if count_to_scheduled_point < elapsed_cycles {
                self.delays
                    .push((scheduled_point, elapsed_cycles - count_to_scheduled_point));
                self.env.incr_pc(count_to_scheduled_point);
            } else {
                self.env.incr_pc(elapsed_cycles);
            }
        }

        // Consume remaining program execution
        self.current_cycle += self.env.end_cycle() - self.env.program_counter();

        Ok(SimulationResult {
            programs: result,
            delays: self.delays,
        })
    }

    fn fresh_job_id(&mut self) -> u32 {
        self.job_counter += 1;
        self.job_counter - 1
    }
}
