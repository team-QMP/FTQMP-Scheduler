use crate::config::SimulationConfig;
use crate::program::{Coordinate, Program, ProgramFormat};
use crate::scheduler::{apply_schedule, JobID, Schedule, Scheduler};

use good_lp::{
    constraint, variable, variables, Expression, ProblemVariables, Solution, SolverModel, Variable,
};

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
struct PackingConfig {
    time_limit: Option<u32>, // in seconds
    size_x: u32,
    size_y: u32,
    size_z: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ScheduleVarsKey {
    i: usize, // an index of a program
    schedule: Schedule,
}

impl ScheduleVarsKey {
    fn new(i: usize, schedule: Schedule) -> Self {
        Self { i, schedule }
    }
}

#[warn(dead_code)]
struct PackingProblem {
    config: PackingConfig,
    vars: ProblemVariables,
    s_vars: HashMap<ScheduleVarsKey, Variable>,
    is_block_present: HashMap<Coordinate, Expression>,
    s_sums: Vec<Expression>,
    total_time: Variable,
}

fn collect_schedule_candidate(
    config: &PackingConfig,
    program: &Program,
) -> Vec<(Schedule, Program)> {
    let mut candidates = Vec::new();
    for r in 0..3 {
        for f in [0, 1] {
            for x in 0..config.size_x {
                for y in 0..config.size_y {
                    for z in 0..config.size_z {
                        let schedule = Schedule::new(x as i32, y as i32, z as i32, r, f == 1);
                        let scheduled = apply_schedule(program, &schedule);
                        let poly = scheduled.polycube().unwrap();
                        if poly.blocks().iter().all(|b| {
                            (b.x as u32) < config.size_x
                                && (b.y as u32) < config.size_y
                                && (b.z as u32) < config.size_z
                        }) {
                            candidates.push((schedule, scheduled));
                        }
                    }
                }
            }
        }
    }

    candidates
}

impl PackingProblem {
    fn new(config: PackingConfig, programs: Vec<Program>) -> Self {
        let mut vars = variables!();
        let mut s_vars = HashMap::new();
        let mut is_block_present = HashMap::new();
        let total_time = vars.add(variable().integer());
        let mut s_sums = Vec::new();
        for (i, program) in programs.iter().enumerate() {
            let candidates = collect_schedule_candidate(&config, program);
            let mut s_sum: Expression = 0.into();
            for (schedule, scheduled) in candidates {
                let s_var = vars.add(variable().binary());
                let s_var_key = ScheduleVarsKey::new(i, schedule);
                s_vars.insert(s_var_key, s_var);
                s_sum += s_var;
                match scheduled.format() {
                    ProgramFormat::Polycube(p) => {
                        for block in p.blocks() {
                            if let Some(exp) = is_block_present.get_mut(block) {
                                *exp += s_var;
                            } else {
                                is_block_present.insert(block.clone(), s_var.into());
                            }
                        }
                    }
                }
            }
            s_sums.push(s_sum);
        }

        Self {
            config,
            vars,
            s_vars,
            s_sums,
            is_block_present,
            total_time,
        }
    }

    fn solve(self) -> Vec<Schedule> {
        let mut problem = self
            .vars
            .minimise(self.total_time)
            .using(good_lp::coin_cbc)
            .with(constraint!(self.total_time >= 0));
        if let Some(time_limit) = self.config.time_limit {
            problem.set_parameter("sec", &format!("{}", time_limit));
        }

        for (pos, is_present) in self.is_block_present {
            problem = problem.with(constraint!(is_present.clone() <= 1));
            problem = problem.with(constraint!(self.total_time >= pos.z * is_present));
        }
        for s_sum in self.s_sums {
            problem = problem.with(constraint!(s_sum == 1));
        }

        let solution = problem.solve().unwrap();
        let mut result = Vec::new();
        for (key, s_var) in self.s_vars {
            if f64::abs(solution.value(s_var) - 1.) <= 1e-8 {
                result.push((key.i, key.schedule.clone()));
            }
        }

        result.sort_by(|(i, _), (j, _)| usize::cmp(i, j));
        result.into_iter().map(|(_, s)| s).collect()
    }
}

// TODO: Currently `LPScheduler` is not supposed to call multiple times.
pub struct LPScheduler {
    program_list: VecDeque<(JobID, Program)>,
    config: SimulationConfig,
}

impl LPScheduler {
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            program_list: VecDeque::new(),
            config,
        }
    }
}

impl Scheduler for LPScheduler {
    fn add_job(&mut self, job_id: JobID, program: Program) {
        self.program_list.push_back((job_id, program));
    }

    fn run(&mut self) -> Vec<(JobID, Schedule)> {
        let worst_zsum = self
            .program_list
            .iter()
            .map(|(_, program)| match program.format() {
                ProgramFormat::Polycube(p) => p.blocks().iter().map(|c| c.z).max().unwrap() as u32,
            })
            .sum();
        let pack_cfg = PackingConfig {
            time_limit: None, // TODO
            size_x: self.config.size_x,
            size_y: self.config.size_y,
            size_z: worst_zsum,
        };
        let programs = self
            .program_list
            .iter()
            .map(|(_, program)| program.clone())
            .collect();
        let problem = PackingProblem::new(pack_cfg, programs);
        let schedules = problem.solve();

        self.program_list
            .iter()
            .map(|(id, _)| *id)
            .zip(schedules)
            .collect()
    }
}

#[cfg(test)]
pub mod test {
    use crate::program::{Polycube, Program, ProgramFormat};
    use crate::scheduler::apply_schedule;
    use crate::scheduler::lp_scheduler::{PackingConfig, PackingProblem};

    #[test]
    fn test_lp() {
        let config = PackingConfig {
            time_limit: Some(60),
            size_x: 4,
            size_y: 3,
            size_z: 8,
        };
        let format = ProgramFormat::Polycube(Polycube::from(&[
            (0, 0, 0),
            (0, 1, 0),
            (0, 2, 0),
            (0, 0, 1),
            (0, 1, 1),
            (0, 2, 1),
            (1, 0, 1),
            (1, 1, 1),
            (1, 2, 1),
            (2, 0, 1),
            (2, 1, 1),
            (2, 2, 1),
            (0, 0, 2),
            (0, 1, 2),
            (0, 2, 2),
            (0, 0, 3),
            (0, 1, 3),
            (0, 2, 3),
            (1, 0, 3),
            (1, 1, 3),
            (1, 2, 3),
            (2, 0, 3),
            (2, 1, 3),
            (2, 2, 3),
        ]));
        let programs: Vec<_> = (0..2).map(|_| Program::new(format.clone())).collect();

        let problem = PackingProblem::new(config.clone(), programs.clone());
        let result = problem.solve();
        assert_eq!(programs.len(), result.len());
        let scheduled: Vec<_> = programs
            .into_iter()
            .zip(result)
            .map(|(p, s)| apply_schedule(&p, &s))
            .collect();
        let mut max_z = 0;
        for i in 0..scheduled.len() {
            let poly1 = scheduled[i].polycube().unwrap();
            for pos in poly1.blocks() {
                assert!(0 <= pos.x && (pos.x as u32) < config.size_x);
                assert!(0 <= pos.y && (pos.y as u32) < config.size_y);
                assert!(0 <= pos.z && (pos.z as u32) < config.size_z);
                max_z = i32::max(max_z, pos.z);
            }
            for j in i + 1..scheduled.len() {
                let poly2 = scheduled[j].polycube().unwrap();
                for pos1 in poly1.blocks() {
                    for pos2 in poly2.blocks() {
                        assert!(pos1 != pos2);
                    }
                }
            }
        }
        assert_eq!(max_z, 4);
    }
}
