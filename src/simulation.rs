use std::collections::BTreeMap;
use anyhow::Result;

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
    env: Environment,
    generator: Box<dyn ProgramGenerator>,
    scheduler: Box<dyn Scheduler>,
    current_time: u32, // TODO: cycle?
    job_que: BTreeMap<u32, Polycube>,
    job_counter: u32,
}

impl Simulator {
    pub fn new(generator: Box<dyn ProgramGenerator>, scheduler: Box<dyn Scheduler>) -> Self {
        Self {
            env: Environment::new(),
            generator,
            scheduler,
            current_time: 0,
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

        // TODO
        unimplemented!()
    }

    fn fresh_job_id(&mut self) -> u32 {
        self.job_counter += 1;
        self.job_counter - 1
    }
}
