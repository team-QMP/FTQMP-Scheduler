use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BinaryHeap};
use std::time::Instant;

use crate::config::SimulationConfig;
use crate::dataset::Dataset;
use crate::environment::Environment;
use crate::error::QMPError;
use crate::event::Event;
use crate::job::{Job, JobID};
use crate::preprocess::{ConvertToCuboid, PreprocessKind, Preprocessor};
use crate::program::Program;
use crate::scheduler::{apply_schedule, Schedule, Scheduler};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuedJob {
    job_id: JobID,
    program: Option<Program>,
    schedule: Schedule,
    requested_time: u64,
    waiting_time: u64,
    turnaround_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub jobs: Vec<IssuedJob>,
    pub total_cycle: u64,
    pub event_log: Vec<Event>,
}

pub struct Simulator {
    config: SimulationConfig,
    env: Environment,
    scheduler: Box<dyn Scheduler>,
    job_que: BinaryHeap<Job>,
    /// The requested but not scheduled job list
    waiting_jobs: BTreeMap<u32, Job>,
    /// The number of cycles elapsed since the start of the simulation.
    simulation_time: u64,
    /// the event log
    event_log: Vec<Event>,
}

impl Simulator {
    pub fn new(config: SimulationConfig, dataset: Dataset, scheduler: Box<dyn Scheduler>) -> Self {
        let preprocessors: Vec<_> = config
            .preprocessor
            .processes
            .iter()
            .map(|kind| match kind {
                PreprocessKind::ConvertToCuboid => {
                    let num_cuboids = config.preprocessor.num_cuboids.map_or(1, |v| v);
                    ConvertToCuboid::new(num_cuboids)
                }
            })
            .collect();
        let job_que: BinaryHeap<_> = dataset
            .requests()
            .into_iter()
            .enumerate()
            .map(|(i, (t, program))| {
                let program = preprocessors
                    .iter()
                    .fold(program.clone(), |p, pp| pp.process(p));
                Job::new(i as JobID, t, program)
            })
            .collect();

        Self {
            env: Environment::new(config.size_x as i32, config.size_y as i32),
            config,
            scheduler,
            job_que,
            waiting_jobs: BTreeMap::new(),
            simulation_time: 0,
            event_log: Vec::new(),
        }
    }

    pub fn run(mut self) -> Result<SimulationResult> {
        let mut result = Vec::new();

        while !self.job_que.is_empty() || !self.waiting_jobs.is_empty() {
            // When the scheduler does not have jobs to be scheduled and there is a job in the job queue,
            // then change the current cycle to the time when the new job is requested.
            if self.waiting_jobs.is_empty()
                && !self.job_que.is_empty()
                && self.job_que.peek().unwrap().requested_time > self.simulation_time
            {
                let forward_cycles =
                    self.job_que.peek().unwrap().requested_time - self.simulation_time;
                self.simulation_time += forward_cycles;
                self.env.incr_pc(forward_cycles);
            }

            while !self.job_que.is_empty()
                && self.job_que.peek().unwrap().requested_time <= self.simulation_time
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

            self.simulation_time += elapsed_cycles;

            // TODO: Estimate a scheduled point
            let scheduled_point = issued_programs
                .iter()
                .map(|(_, schedule)| schedule.z as u64)
                .min()
                .unwrap_or(u64::MAX);

            for (job_id, schedule) in issued_programs {
                if (schedule.z as u64) < self.env.program_counter() {
                    return Err(QMPError::ViolateTimingConstraint.into());
                }
                let job = self
                    .waiting_jobs
                    .get(&job_id)
                    .ok_or_else(|| QMPError::invalid_job_id(job_id))?;
                let scheduled_program = apply_schedule(&job.program, &schedule);
                if !self.env.issue_program(&scheduled_program) {
                    return Err(QMPError::invalid_schedule_error(job.clone(), schedule));
                }

                let waiting_time = self.simulation_time - job.requested_time;
                let turnaround_time = waiting_time + scheduled_program.burst_time();
                let issued_job = IssuedJob {
                    job_id: job.id,
                    program: if self.config.no_output_program {
                        None
                    } else {
                        Some(scheduled_program)
                    },
                    schedule,
                    requested_time: job.requested_time,
                    waiting_time,
                    turnaround_time,
                };
                result.push(issued_job);
                self.waiting_jobs.remove(&job_id);
            }

            // When the program point specified by the scheduler (= minimum z position of schedules) is reached, program execution is stopped until the result is returned.
            let count_to_scheduled_point = scheduled_point - self.env.program_counter();
            if count_to_scheduled_point < elapsed_cycles {
                self.log_event(Event::suspend_exec(
                    scheduled_point,
                    elapsed_cycles - count_to_scheduled_point,
                ));
                self.env.incr_pc(count_to_scheduled_point);
            } else {
                self.env.incr_pc(elapsed_cycles);
            }
        }

        // Consume remaining program execution
        self.simulation_time += self.env.end_cycle() - self.env.program_counter();

        Ok(SimulationResult {
            jobs: result,
            total_cycle: self.simulation_time,
            event_log: self.event_log,
        })
    }

    pub fn log_event(&mut self, event: Event) {
        self.event_log.push(event)
    }
}
