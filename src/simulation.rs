use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::config::SimulationConfig;
use crate::dataset::Dataset;
use crate::environment::Environment;
use crate::error::QMPError;
use crate::event::{Event, EventQueue, EventType};
use crate::job::{Job, JobID, JobStatus};
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
    /// The job list
    job_list: Vec<Job>,
    /// The number of cycles elapsed since the start of the simulation.
    simulation_time: u64,
    /// the event queue
    event_que: EventQueue,
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

        let mut job_list = Vec::new();
        let mut event_que = EventQueue::new();
        for (i, (t, p)) in dataset.requests().into_iter().enumerate() {
            let p = preprocessors
                .iter()
                .fold(p.clone(), |p, proc| proc.process(p));
            let job = Job::new(i as JobID, t, p.clone());
            job_list.push(job);
            event_que.add_event(Event::request_job(t, i as JobID));
        }

        Self {
            env: Environment::new(config.size_x as i32, config.size_y as i32),
            config,
            scheduler,
            job_list,
            simulation_time: 0,
            event_que: EventQueue::new(),
            event_log: Vec::new(),
        }
    }

    pub fn run(mut self) -> Result<SimulationResult> {
        let mut result = Vec::new();

        while let Some(event) = self.event_que.pop() {
            let event_time = event.event_time();
            assert!(event_time >= self.simulation_time);

            let elapsed_cycle = event_time - self.simulation_time;
            self.simulation_time = event_time;
            self.env.proceed_cycle(elapsed_cycle);

            match event.event_type() {
                EventType::RequestJob { job_id } => {
                    let job_id = *job_id as usize;
                    self.job_list[job_id].update_status(JobStatus::Waiting);
                    self.scheduler.add_job(self.job_list[job_id].clone());
                }
                EventType::StartScheduling => {
                    let start = Instant::now();
                    let issued_programs = self.scheduler.run(&self.env);
                    let elapsed_cycles = start
                        .elapsed()
                        .as_micros()
                        .div_ceil(self.config.micro_sec_per_cycle.into())
                        as u64;

                    let scheduled_point = issued_programs
                        .iter()
                        .map(|(_, schedule)| schedule.z as u64)
                        .min()
                        .unwrap_or(u64::MAX);

                    for (job_id, schedule) in issued_programs {
                        let job = &self.job_list[job_id as usize];
                        if (schedule.z as u64) < self.env.global_pc() {
                            return Err(QMPError::ViolateTimingConstraint.into());
                        }
                        if job_id as usize >= self.job_list.len()
                            || job.status() != &JobStatus::Waiting
                        {
                            return Err(QMPError::invalid_job_id(job_id));
                        }
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
                        self.job_list[job_id as usize].update_status(JobStatus::Scheduled);
                    }

                    // When the program point specified by the scheduler (= minimum z position of schedules) is reached,
                    // program execution is stopped until the result is returned.
                    let cycle_count_to_scheduled_point = scheduled_point - self.env.global_pc();
                    if cycle_count_to_scheduled_point < elapsed_cycles {
                        self.event_que.add_event(Event::suspend_exec(
                            scheduled_point,
                            elapsed_cycles - cycle_count_to_scheduled_point,
                        ));
                    }
                    // We don't update `simulation_time` by elapsed_cycles here because the scheduler
                    // runs concurrently.

                    self.event_que.add_event(Event::start_scheduling(
                        self.simulation_time + elapsed_cycle,
                    ));
                }
                EventType::SuspendExec { duration } => {
                    self.env.add_suspend_cycle(*duration);
                }
            }
        }

        // All jobs must be either running or finished
        assert!(self
            .job_list
            .iter()
            .all(|job| job.status() != &JobStatus::Waiting));

        // Consume remaining program execution
        self.simulation_time += self.env.remaining_cycles();

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
