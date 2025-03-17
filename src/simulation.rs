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
    pub event_log: Vec<Event>,
    pub jobs: Vec<IssuedJob>,
    pub total_cycle: u64,
    /// The summation of $z$ length of programs
    pub z_sum: u64,
    pub max_z: u64,
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
        // add initial scheduling point
        event_que.add_event(Event::start_scheduling(0));

        //if config.enable_defrag {
        //    let init_defrag_point = config.defrag_interval.unwrap();
        //    event_que.add_event(Event::defragmentation(init_defrag_point));
        //}

        Self {
            env: Environment::new(config.size_x as i32, config.size_y as i32),
            config,
            scheduler,
            job_list,
            simulation_time: 0,
            event_que,
            event_log: Vec::new(),
        }
    }

    pub fn run(mut self) -> Result<SimulationResult> {
        let mut result = Vec::new();

        let mut z_sum = 0;
        let mut prev_defrag_point = 0;

        while let Some(event) = self.event_que.pop() {
            // If all jobs have been scheduled, we ignore the remaining event because they does not
            // affect the simulation result.
            if self
                .job_list
                .iter()
                .all(|job| job.status() != &JobStatus::Waiting)
            {
                break;
            }
            tracing::debug!("Event occur: {:?}", event);
            let event_time = event.event_time();
            assert!(event_time >= self.simulation_time);

            self.env.advance_by(event_time - self.simulation_time);
            self.simulation_time = event_time;
            assert!(self.env.current_time() == self.simulation_time);

            match event.event_type() {
                EventType::RequestJob { job_id } => {
                    let job_id = *job_id as usize;
                    self.job_list[job_id].update_status(JobStatus::Waiting);
                    z_sum += self.job_list[job_id].total_execution_cycle();
                    self.scheduler.add_job(self.job_list[job_id].clone());
                }
                EventType::StartScheduling => {
                    if self.config.enable_defrag {
                        self.env.defrag();
                    }

                    let start = Instant::now();
                    let issued_programs = self.scheduler.run(&self.env);
                    let elapsed_cycles = start
                        .elapsed()
                        .as_micros()
                        .div_ceil(self.config.micro_sec_per_cycle.into())
                        as u64;
                    let has_scheduled = !issued_programs.is_empty();

                    // If the current job que is empty, then the scheduler waits until the next
                    // event will occur
                    if !has_scheduled {
                        let next_scheduling_time = self
                            .event_que
                            .next_event_time()
                            .expect("there must be remaining job");
                        self.event_que
                            .add_event(Event::start_scheduling(next_scheduling_time));

                        continue;
                    }

                    tracing::debug!("Scheduling took {} cycles", elapsed_cycles);

                    let scheduled_point = issued_programs
                        .iter()
                        .map(|(_, schedule)| schedule.z as u64)
                        .min()
                        .expect("At least one job must be scheduled here");

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
                    self.env
                        .suspend_at(scheduled_point, self.simulation_time + elapsed_cycles);
                    // We don't update `simulation_time` by elapsed_cycles here because the scheduler
                    // runs concurrently.

                    // If there are waiting jobs, prepare next scheduling
                    if self
                        .job_list
                        .iter()
                        .any(|job| job.status() == &JobStatus::Waiting)
                    {
                        let next_scheduling_time = self.simulation_time + elapsed_cycles;
                        self.event_que
                            .add_event(Event::start_scheduling(next_scheduling_time));
                    }
                }
                EventType::Defragmentation => {
                    let defrag_interval = self.config.defrag_interval.unwrap();
                    if prev_defrag_point + defrag_interval <= self.env.global_pc() {
                        self.env.defrag();
                        prev_defrag_point = self.env.global_pc();
                    }
                    self.event_que.add_event(Event::defragmentation(
                        self.simulation_time + self.config.defrag_interval.unwrap(),
                    ))
                }
            }
        }

        // All jobs must be either running or finished
        assert!(self
            .job_list
            .iter()
            .all(|job| job.status() != &JobStatus::Waiting));

        self.env.validate();

        // Consume remaining program execution
        tracing::debug!("#remaining cycles = {}", self.env.remaining_cycles());
        self.simulation_time += self.env.remaining_cycles();

        tracing::debug!("final PC = {}", self.env.end_pc());

        Ok(SimulationResult {
            event_log: self.event_log,
            jobs: result,
            total_cycle: self.simulation_time,
            z_sum,
            max_z: self.env.end_pc(),
        })
    }

    pub fn log_event(&mut self, event: Event) {
        self.event_log.push(event)
    }
}
