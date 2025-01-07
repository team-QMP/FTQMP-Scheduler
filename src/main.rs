use clap::Parser;
use std::path::PathBuf;

use anyhow::Result;

use qmp_scheduler::config::SimulationConfig;
use qmp_scheduler::generator::TestGenerator;
use qmp_scheduler::scheduler::{GreedyScheduler, LPScheduler, Scheduler};
use qmp_scheduler::simulation::Simulator;

#[derive(Debug, Clone, PartialEq, clap::ValueEnum)]
pub enum SchedulerKind {
    Greedy,
    LP,
}

#[derive(Debug, Clone, PartialEq, clap::ValueEnum)]
pub enum GeneratorKind {
    Test,
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    config_path: PathBuf,

    #[arg(short, long, default_value = "result.json")]
    output_file: PathBuf,

    #[arg(long)]
    scheduler: SchedulerKind,

    #[arg(long)]
    generator: GeneratorKind,
}

fn main() -> Result<()> {
    use std::io::prelude::Write;

    let args = Args::parse();

    let config = SimulationConfig::from_json_file(args.config_path.clone())?;
    let scheduler: Box<dyn Scheduler> = match args.scheduler {
        SchedulerKind::Greedy => Box::new(GreedyScheduler::new(config.clone())),
        SchedulerKind::LP => Box::new(LPScheduler::new(config.clone())),
    };
    let generator = match args.generator {
        GeneratorKind::Test => Box::new(TestGenerator::new()),
    };

    let mut simulator = Simulator::new(config, generator, scheduler);

    let result = simulator.run()?;

    let mut output_file = std::fs::File::create(args.output_file)?;
    output_file.write_all(serde_json::to_string(&result)?.as_bytes())?;

    Ok(())
}
