use std::collections::BTreeMap;
use std::time::Instant;
use anyhow::Result;

use crate::config::SimulationConfig;
use crate::scheduler::Scheduler;
use crate::ds::polycube::{Coordinate, Polycube};
use crate::environment::Environment;
use crate::generator::ProgramGenerator;

#[derive(Debug, Clone)]
struct SimulationResult {
    blocks: Vec<Coordinate>
}

#[derive(Debug, Clone)]
struct Job {
    added_time: u32,
    program: Polycube,
}

impl Job {
    pub fn new(program: Polycube, added_time: u32) -> Self {
        Job {
            program,
            added_time,
        }
    }
}

struct Simulator {
    config: SimulationConfig,
    env: Environment,
    generator: Box<dyn ProgramGenerator>,
    scheduler: Box<dyn Scheduler>,
    current_cycle: u128, // TODO: time (e.g., ns)?
    job_que: BTreeMap<u32, Polycube>,
    job_counter: u32,
}

impl Simulator {
    pub fn new(config: SimulationConfig, generator: Box<dyn ProgramGenerator>, scheduler: Box<dyn Scheduler>) -> Self {
        Self {
            config,
            env: Environment::new(),
            generator,
            scheduler,
            current_cycle: 0,
            job_que: BTreeMap::new(),
            job_counter: 0,
        }
    }

    pub fn run(&mut self) -> Result<SimulationResult> {
        // Generate "all" programs first
        let program_num = 3;
        for prog in self.generator.generate(program_num) {
            let job_id = self.fresh_job_id();
            self.job_que.insert(job_id, prog);
        }

        while self.job_que.len() != 0 {
            // TODO: Notify added programs to scheduler
            // ...

            // TODO: How to estimate the execution time of the scheduler?
            let start = Instant::now();
            let issued_programs = self.scheduler.run();
            let elapsed = start.elapsed().as_micros();
            let elapsed_cycles = (elapsed + self.config.micro_sec_per_cycle - 1) / self.config.micro_sec_per_cycle;

            // TODO: Remove issued programs from the job queue

            self.current_cycle += elapsed_cycles;
        }

        // TODO: return simulation result
        unimplemented!()
    }

    fn fresh_job_id(&mut self) -> u32 {
        self.job_counter += 1;
        self.job_counter - 1
    }
}
