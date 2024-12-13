use std::collections::{BinaryHeap, BTreeMap};
use std::time::Instant;
use std::cmp::Ordering;
use anyhow::Result;

use crate::config::SimulationConfig;
use crate::error::QMPError;
use crate::scheduler::{apply_schedule, Scheduler};
use crate::ds::program::Program;
use crate::ds::polycube::Polycube;
use crate::environment::Environment;
use crate::generator::ProgramGenerator;

#[derive(Debug, Clone)]
pub struct SimulationResult {
    programs: Vec<Program>
}

pub type JobID = u32;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Job {
    id: JobID,
    added_time: u128,
    program: Program,
}

impl Job {
    pub fn new(id: JobID, added_time: u128, program: Program) -> Self {
        Job {
            id,
            added_time,
            program,
        }
    }
}

impl Ord for Job {
    fn cmp(&self, other: &Self) -> Ordering {
        other.added_time.cmp(&self.added_time)
    }
}

impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
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
    pub fn new(config: SimulationConfig, generator: Box<dyn ProgramGenerator>, scheduler: Box<dyn Scheduler>) -> Self {
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
            self.job_que.insert(job_id, Job::new(job_id, t, program.clone()));
            self.future_job_que.push(Job::new(job_id, t, program));
        }

        let mut result = Vec::new(); // TODO

        while self.job_que.len() != 0 && self.future_job_que.len() != 0 {
            while self.future_job_que.len() != 0
                && self.future_job_que.peek().unwrap().added_time <= self.current_cycle
            {
                let job = self.future_job_que.pop().unwrap();
                self.scheduler.add_job(job.id, job.program);
            }

            // TODO: How to estimate the execution time of the scheduler?
            let start = Instant::now();
            let issued_programs = self.scheduler.run();
            let elapsed = start.elapsed().as_micros();
            let elapsed_cycles = (elapsed + self.config.micro_sec_per_cycle - 1) / self.config.micro_sec_per_cycle;

            for (job_id, schedule) in issued_programs {
                let job = self.job_que.get(&job_id).ok_or_else(|| {
                    QMPError::invalid_job_id(job_id)
                })?;
                let scheduled_program = apply_schedule(&job.program, &schedule);
                if !self.env.insert_program(&scheduled_program) {
                    return Err(QMPError::invalid_schedule_error(job_id, schedule))
                }
                result.push(scheduled_program);
                self.job_que.remove(&job_id);
            }

            self.current_cycle += elapsed_cycles;
        }

        Ok(SimulationResult {
            programs: result
        })
    }

    fn fresh_job_id(&mut self) -> u32 {
        self.job_counter += 1;
        self.job_counter - 1
    }
}
