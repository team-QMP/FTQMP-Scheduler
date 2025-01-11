use crate::config::SimulationConfig;
use crate::job::Job;
use crate::program::{Coordinate, Cuboid, Program, ProgramFormat};
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

enum PackingProblem {
    Polycube(PolycubePackingProblem),
    Cuboid(CuboidPackingProblem),
}

impl PackingProblem {
    pub fn solve(self) -> Vec<Schedule> {
        match self {
            PackingProblem::Polycube(problem) => problem.solve(),
            PackingProblem::Cuboid(problem) => problem.solve(),
        }
    }
}

#[warn(dead_code)]
struct PolycubePackingProblem {
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

impl PolycubePackingProblem {
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
                    _ => unimplemented!(),
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

/// The specialized LP problem when all programs represented by cuboids.
/// constant values:
///   * X, Y                = the chip size (constant value)
///   * sx[i], sy[i], sz[i] = the size of i-th cuboid
///   * Z                   = sum_i sz[i]
/// variables:
///   * a[i][j] := (x pos of i-th cuboid) > (x pos of j-th cuboid) (binary)
///   * b[i][j] := (y pos of i-th cuboid) > (y pos of j-th cuboid) (binary)
///   * c[i][j] := (z pos of i-th cuboid) > (z pos of j-th cuboid) (binary)
///   * x[i], y[i], z[i] := the position of i-th cuboid
///   * v := the objective value
/// minimize v
/// s.t.
///   * a[i][j] + a[j][i] + b[i][j] + b[j][i] + c[i][j] + c[j][i] >= 1
///   * x[i] - x[j] + X * a[i][j] <= X - sx[i] for all i, j
///   * y[i] - y[j] + Y * b[i][j] <= Y - sy[i] for all i, j
///   * z[i] - z[j] + Z * c[i][j] <= Z - sz[i] for all i, j
///   * x[i] + sx[i] <= X
///   * y[i] + sy[i] <= Y
///   * z[i] + sz[i] <= Z
///   * z[i] + sz[i] <= v
#[warn(dead_code)]
struct CuboidPackingProblem {
    config: PackingConfig,
    n: usize, // #(cuboids)
    vars: ProblemVariables,
    x: Vec<Variable>,
    y: Vec<Variable>,
    z: Vec<Variable>,
    a: Vec<Vec<Option<Variable>>>, // i == j iff None
    b: Vec<Vec<Option<Variable>>>,
    c: Vec<Vec<Option<Variable>>>,
    v: Variable,
    size: Vec<[usize; 3]>,
}

// TODO: Consider rotations
impl CuboidPackingProblem {
    /// The positions of all cuboids must be (0, 0, 0)
    pub fn new(config: PackingConfig, cuboids: Vec<Cuboid>) -> Self {
        assert!(!cuboids.is_empty());
        assert!(cuboids
            .iter()
            .all(|c| { c.pos().x == 0 && c.pos().y == 0 && c.pos().z == 0 }));
        let mut vars = variables!();
        let n = cuboids.len();
        let x: Vec<_> = (0..n).map(|_| vars.add(variable().integer())).collect();
        let y: Vec<_> = (0..n).map(|_| vars.add(variable().integer())).collect();
        let z: Vec<_> = (0..n).map(|_| vars.add(variable().integer())).collect();
        let a: Vec<Vec<_>> = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| {
                        if i == j {
                            None
                        } else {
                            Some(vars.add(variable().binary()))
                        }
                    })
                    .collect()
            })
            .collect();
        let b: Vec<Vec<_>> = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| {
                        if i == j {
                            None
                        } else {
                            Some(vars.add(variable().binary()))
                        }
                    })
                    .collect()
            })
            .collect();
        let c: Vec<Vec<_>> = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| {
                        if i == j {
                            None
                        } else {
                            Some(vars.add(variable().binary()))
                        }
                    })
                    .collect()
            })
            .collect();
        let v = vars.add_variable();

        Self {
            config,
            n,
            vars,
            x,
            y,
            z,
            a,
            b,
            c,
            v,
            size: cuboids
                .iter()
                .map(|c| [c.size_x(), c.size_y(), c.size_z()])
                .collect(),
        }
    }

    pub fn solve(mut self) -> Vec<Schedule> {
        let mut problem = self.vars.minimise(self.v).using(good_lp::coin_cbc);

        if let Some(time_limit) = self.config.time_limit {
            problem.set_parameter("sec", &format!("{}", time_limit));
        }

        let max_x = self.config.size_x as i32; // X
        let max_y = self.config.size_y as i32; // Y
        let max_z = self.config.size_z as i32; // Z

        for i in 0..self.n {
            let [size_xi, size_yi, size_zi] = self.size[i];
            let (size_xi, size_yi, size_zi) = (size_xi as i32, size_yi as i32, size_zi as i32);
            let xi = self.x[i];
            let yi = self.y[i];
            let zi = self.z[i];
            for j in 0..self.n {
                if i == j {
                    continue;
                }
                let aij = self.a[i][j].unwrap();
                let aji = self.a[j][i].unwrap();
                let bij = self.b[i][j].unwrap();
                let bji = self.b[j][i].unwrap();
                let cij = self.c[i][j].unwrap();
                let cji = self.c[j][i].unwrap();
                let xj = self.x[j];
                let yj = self.y[j];
                let zj = self.z[j];

                if i < j {
                    problem = problem.with(constraint!(aij + aji + bij + bji + cij + cji >= 1));
                }
                problem = problem
                    .with(constraint!(xi - xj + max_x * aij <= max_x - size_xi))
                    .with(constraint!(yi - yj + max_y * bij <= max_y - size_yi))
                    .with(constraint!(zi - zj + max_z * cij <= max_z - size_zi));
            }
            problem = problem
                .with(constraint!(0 <= xi))
                .with(constraint!(xi + size_xi <= max_x))
                .with(constraint!(0 <= yi))
                .with(constraint!(yi + size_yi <= max_y))
                .with(constraint!(0 <= zi))
                .with(constraint!(zi + size_zi <= max_z))
                .with(constraint!(zi + size_zi <= self.v));
        }

        let solution = problem.solve().unwrap();
        (0..self.n)
            .map(|i| {
                let xi = solution.value(self.x[i]).round() as i32;
                let yi = solution.value(self.y[i]).round() as i32;
                let zi = solution.value(self.z[i]).round() as i32;
                Schedule::new(xi, yi, zi, 0, false)
            })
            .collect()
    }
}

// TODO: Currently `LPScheduler` is not supposed to call multiple times.
pub struct LPScheduler {
    job_list: VecDeque<Job>,
    config: SimulationConfig,
}

impl LPScheduler {
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            job_list: VecDeque::new(),
            config,
        }
    }
}

impl Scheduler for LPScheduler {
    fn add_job(&mut self, job: Job) {
        self.job_list.push_back(job);
    }

    fn run(&mut self) -> Vec<(JobID, Schedule)> {
        let worst_zsum = self
            .job_list
            .iter()
            .map(|job| match job.program.format() {
                ProgramFormat::Polycube(p) => {
                    p.blocks().iter().map(|c| c.z).max().unwrap() as u32 + 1
                }
                ProgramFormat::Cuboid(c) => {
                    assert!(c.pos().z == 0);
                    c.size_z() as u32
                }
            })
            .sum();
        let pack_cfg = PackingConfig {
            time_limit: None, // TODO
            size_x: self.config.size_x,
            size_y: self.config.size_y,
            size_z: worst_zsum,
        };
        // TODO: Batch processing
        let programs: Vec<_> = self
            .job_list
            .iter()
            .map(|job| job.program.clone())
            .collect();
        let problem = if programs.iter().all(|p| p.is_polycube()) {
            PackingProblem::Polycube(PolycubePackingProblem::new(pack_cfg, programs))
        } else if programs.iter().all(|p| p.is_cuboid()) {
            let cuboids = programs
                .into_iter()
                .map(|p| p.cuboid().unwrap().clone())
                .collect();
            PackingProblem::Cuboid(CuboidPackingProblem::new(pack_cfg, cuboids))
        } else {
            unimplemented!()
        };
        let schedules = problem.solve();

        self.job_list
            .iter()
            .map(|job| job.id)
            .zip(schedules)
            .collect()
    }
}

#[cfg(test)]
pub mod test {
    use crate::program::{Coordinate, Cuboid, Polycube, Program, ProgramFormat};
    use crate::scheduler::lp_scheduler::{PackingConfig, PackingProblem};
    use crate::scheduler::{apply_schedule, Schedule};

    //#[test]
    fn test_lp_polycube() {
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

    #[test]
    fn test_lp_cuboid() {
        use crate::scheduler::lp_scheduler::CuboidPackingProblem;

        let cuboid_1x1x1 = Cuboid::new(Coordinate::new(0, 0, 0), 1, 1, 1);
        let cuboid_1x2x1 = Cuboid::new(Coordinate::new(0, 0, 0), 1, 2, 1);
        let cuboid_1x2x2 = Cuboid::new(Coordinate::new(0, 0, 0), 1, 2, 2);
        let cuboids = vec![
            cuboid_1x2x2,
            cuboid_1x1x1.clone(),
            cuboid_1x2x1,
            cuboid_1x1x1,
        ];

        let config = PackingConfig {
            time_limit: Some(60),
            size_x: 2,
            size_y: 2,
            size_z: 2,
        };

        let problem = CuboidPackingProblem::new(config.clone(), cuboids.clone());
        let schedule = problem.solve();
        for i in 0..cuboids.len() {
            let xi = schedule[i].x;
            let yi = schedule[i].y;
            let zi = schedule[i].z;
            let size_xi = cuboids[i].size_x() as i32;
            let size_yi = cuboids[i].size_y() as i32;
            let size_zi = cuboids[i].size_z() as i32;
            assert!(0 <= xi && xi + size_xi <= config.size_x as i32);
            assert!(0 <= yi && yi + size_yi <= config.size_y as i32);
            assert!(0 <= zi && zi + size_zi <= config.size_z as i32);
            for j in i + 1..cuboids.len() {
                let xj = schedule[j].x;
                let yj = schedule[j].y;
                let zj = schedule[j].z;
                let size_xj = cuboids[j].size_x() as i32;
                let size_yj = cuboids[j].size_y() as i32;
                let size_zj = cuboids[j].size_z() as i32;
                let is_overlap_x = !(xi + size_xi <= xj || xj + size_xj <= xi);
                let is_overlap_y = !(yi + size_yi <= yj || yj + size_yj <= yi);
                let is_overlap_z = !(zi + size_zi <= zj || zj + size_zj <= zi);
                assert!(!is_overlap_x || !is_overlap_y || !is_overlap_z);
            }
        }
    }
}
