use good_lp::solvers::coin_cbc::CoinCbcProblem;
use good_lp::{constraint, default_solver, variable, variables, Expression, ProblemVariables, Solution, SolverModel, Variable};

use crate::ds::polycube::Coordinate;
use crate::ds::program::{Program, ProgramFormat};
use crate::scheduler::{Schedule, apply_schedule};
//use crate::scheduler::{JobID, Scheduler, Schedule, apply_schedule};
//use crate::environment::Environment;
//use crate::config::SimulationConfig;

use std::collections::HashMap;
//use std::collections::VecDeque;

// #var_num := 導入される変数の数
// #const_num := 導入される制約式の数
//
// - (CONST) X, Y, Z := 配置可能最大座標
// - (VAR) s(i,r,f,x,y,z) := i 番目の polycube を rotate r, flip f で x, y, z にスケジュールするかどうか
//   - For all i, sum[r, f, x, y, z] s(i,r,f,x,y,z) = 1
//   - #var_num == 考えられうるスケジューリングの候補数
//   - #const_num == プログラム数
// - (CONST) P(i,r,f,x,y,z) := s(i,r,f,x,y,z) を選んだ場合のポリキューブ
//   - 各ブロック p in P(i,r,f,x,y,z) が 0 <= p_x <= X (Y, Z も同じ) となるように s は制限しておく
// - (VAR) T[i,x,y,z] := (x,y,z) に i 番目のポリキューブのブロックが存在するか 
//   - T[i,x,y,z] := sum[r, f, (x',y',z') s.t. (x,y,z) in P(i,r,f,x',y',z')] s(i,r,f,x',y',z')
//   - #var_num == #const_num == プログラム数 * 空間サイズ
// - (VAR) U := 目的変数 (total execution time)
//   - #var_num == 1
//   - 案1: forall i,r,f,x,y,z. U >= max{p_z | p in P(i,r,f,x,y,z)} * s(i,r,f,x,x,y)
//     - #const_num == プログラムの数 * スケジューリング候補数(=空間サイズ)
//   - 案2: forall (x,y,z) in [0,X]*..*[0,Z]. already(x,y,z) + sum[i] T(i,x,y,z)
//     - #const_num == 空間サイズ
//     - こっちのほうがいいかも

#[derive(Debug, Clone)]
struct PackingConfig {
    time_limit: Option<u32>, // in seconds
    size_x: i32,
    size_y: i32,
    size_z: i32
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ScheduleVarsKey {
    i: usize, // an index of a program
    schedule: Schedule,
}

impl ScheduleVarsKey {
    fn new(i: usize, schedule: Schedule) -> Self {
        Self {
            i,
            schedule
        }
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

fn collect_schedule_candidate(config: &PackingConfig, program: &Program) -> Vec<(Schedule, Program)> {
    let mut candidates = Vec::new();
    for r in 0..3 {
        for f in [0, 1] {
            for x in 0..config.size_x {
                for y in 0..config.size_y {
                    for z in 0..config.size_z {
                        let schedule = Schedule::new(x, y, z, r, f == 1);
                        let scheduled= apply_schedule(&program, &schedule);
                        let poly = scheduled.polycube().unwrap();
                        if poly.blocks().iter().all(|b| { b.x < config.size_x && b.y < config.size_y && b.z < config.size_z }) {
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
            total_time
        }
    }

    fn solve(self) -> Vec<Schedule> {
        let mut problem = self.vars.minimise(self.total_time)
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

// pub struct LPScheduler {
//     program_list: VecDeque<(JobID, Program)>,
//     env: Environment,
//     config: SimulationConfig,
// }
// 
// impl LPScheduler {
//     pub fn new(config: SimulationConfig) -> Self {
//         Self {
//             program_list: VecDeque::new(),
//             env: Environment::new(config.size_x as i32, config.size_y as i32),
//             config,
//         }
//     }
// }
// 
// impl Scheduler for LPScheduler {
//     fn add_job(&mut self, job_id: JobID, program: Program) {
//         unimplemented!()
//     }
// 
//     fn run(&mut self) -> Vec<(JobID, Schedule)> {
//         unimplemented!()
//     }
// }


#[cfg(test)]
pub mod test {
    use crate::ds::program::{Program, ProgramFormat};
    use crate::ds::polycube::Polycube;
    use crate::scheduler::apply_schedule;
    use crate::scheduler::lp_scheduler::{PackingConfig, PackingProblem};

    #[test]
    fn test_lp() {
        let config = PackingConfig {
            time_limit: Some(60),
            size_x: 4,
            size_y: 3,
            size_z: 100
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
            (2, 2, 3)
        ]));
        let programs: Vec<_> = (0..2).map(|_| Program::new(format.clone())).collect();

        let problem = PackingProblem::new(config.clone(), programs.clone());
        let result = problem.solve();
        assert_eq!(programs.len(), result.len());
        let scheduled: Vec<_> = programs.into_iter()
            .zip(result)
            .map(|(p, s)| {
                apply_schedule(&p, &s)
            }).collect();
        let mut max_z = 0;
        for i in 0..scheduled.len() {
            let poly1 = scheduled[i].polycube().unwrap();
            for pos in poly1.blocks() {
                assert!(0 <= pos.x && pos.x < config.size_x);
                assert!(0 <= pos.y && pos.y < config.size_y);
                assert!(0 <= pos.z && pos.z < config.size_z);
                max_z = i32::max(max_z, pos.z);
            }
            for j in i+1..scheduled.len() {
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
