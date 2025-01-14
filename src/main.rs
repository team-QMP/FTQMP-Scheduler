use clap::Parser;
use std::path::PathBuf;

use anyhow::Result;

use qmp_scheduler::config::SimulationConfig;
use qmp_scheduler::dataset::Dataset;
use qmp_scheduler::scheduler::{GreedyScheduler, LPScheduler, Scheduler, SchedulerKind};
use qmp_scheduler::simulation::Simulator;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    config_path: PathBuf,

    #[arg(short, long, default_value = "result.json")]
    output_file: PathBuf,

    #[arg(short, long)]
    dataset_file: PathBuf,
}

fn main() -> Result<()> {
    use std::io::prelude::Write;

    let args = Args::parse();

    let config = SimulationConfig::from_toml(args.config_path.clone())?;
    let dataset = Dataset::from_json_file(args.dataset_file)?;
    let scheduler: Box<dyn Scheduler> = match config.scheduler.kind {
        SchedulerKind::Greedy => Box::new(GreedyScheduler::new(config.clone())),
        SchedulerKind::LP => Box::new(LPScheduler::new(config.clone())),
    };

    let simulator = Simulator::new(config, dataset, scheduler);

    let result = simulator.run()?;

    let mut output_file = std::fs::File::create(args.output_file)?;
    output_file.write_all(serde_json::to_string(&result)?.as_bytes())?;

    Ok(())
}
