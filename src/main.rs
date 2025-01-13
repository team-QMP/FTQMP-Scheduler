use clap::Parser;
use std::path::PathBuf;

use anyhow::Result;

use qmp_scheduler::config::SimulationConfig;
use qmp_scheduler::generator::{GeneratorKind, TestGenerator};
use qmp_scheduler::scheduler::{SchedulerKind, GreedyScheduler, LPScheduler, Scheduler};
use qmp_scheduler::simulation::Simulator;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    config_path: PathBuf,

    #[arg(short, long, default_value = "result.json")]
    output_file: PathBuf,
}

fn main() -> Result<()> {
    use std::io::prelude::Write;

    let args = Args::parse();

    let config = SimulationConfig::from_toml(args.config_path.clone())?;
    let generator = match config.generator.kind {
        GeneratorKind::Test => Box::new(TestGenerator::new()),
    };
    let scheduler: Box<dyn Scheduler> = match config.scheduler.kind {
        SchedulerKind::Greedy => Box::new(GreedyScheduler::new(config.clone())),
        SchedulerKind::LP => Box::new(LPScheduler::new(config.clone())),
    };

    let mut simulator = Simulator::new(config, generator, scheduler);

    let result = simulator.run()?;

    let mut output_file = std::fs::File::create(args.output_file)?;
    output_file.write_all(serde_json::to_string(&result)?.as_bytes())?;

    Ok(())
}
