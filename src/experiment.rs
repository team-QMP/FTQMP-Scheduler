#[derive(Debug, Clone)]
struct ExperimentResult {
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

#[derive(Debug, Clone)]
struct Experiment {
    env: Environment,
    generator: ProgramGenerator,
    scheduler: Scheduler,
    current_time: u32, // TODO: cycle?
}

impl Experiment {
    pub fn new(env: Environment, generator: ProgramGenerator, scheduler: Scheduler) -> Self {
        Self {
            env,
            generator,
            scheduler,
            current_time: 0,
        }
    }

    // TODO
    // ...
    
    pub fn run(&mut self) -> Result<ExperimentResult> {
        unimplemented!()
    }
}
