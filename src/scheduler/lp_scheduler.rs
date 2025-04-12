use std::time::Instant;

use crate::config::SimulationConfig;
use crate::environment::Environment;
use crate::job::Job;
use crate::program::{Coordinate, Cuboid, Program, ProgramFormat};
use crate::scheduler::{apply_schedule, JobID, Schedule, Scheduler};

#[cfg(not(feature = "with-cplex"))]
use good_lp::solvers::coin_cbc::CoinCbcProblem;
#[cfg(feature = "with-cplex")]
use good_lp::solvers::cplex::CPLEXProblem;
use good_lp::variable::UnsolvedProblem;
use good_lp::{
    constraint, variable, variables, Expression, ProblemVariables, Solution, SolverModel, Variable,
};

use std::collections::{HashMap, VecDeque};

pub struct LPSolverWrapper {
    #[cfg(not(feature = "with-cplex"))]
    problem: CoinCbcProblem,
    #[cfg(feature = "with-cplex")]
    problem: CPLEXProblem,
}

impl LPSolverWrapper {
    pub fn new(problem: UnsolvedProblem, time_limit: Option<u32>) -> Self {
        #[cfg(not(feature = "with-cplex"))]
        let problem = if let Some(time_limit) = time_limit {
            let mut problem = problem.using(good_lp::coin_cbc);
            problem.set_parameter("sec", &format!("{}", time_limit));
            problem
        } else {
            problem.using(good_lp::coin_cbc)
        };

        #[cfg(feature = "with-cplex")]
        let problem = {
            if let Some(time_limit) = time_limit {
                let sec = std::time::Duration::new(time_limit.into(), 0);
                let time_limit = cplex_rs::parameters::TimeLimit(sec);
                let mut cplex_env = cplex_rs::Environment::new().expect("");
                cplex_env.set_parameter(time_limit).unwrap(); // TODO
                good_lp::solvers::cplex::cplex_with_env(problem, cplex_env)
            } else {
                problem.using(good_lp::solvers::cplex::cplex)
            }
        };

        Self { problem }
    }

    pub fn with(self, c: good_lp::Constraint) -> Self {
        Self {
            problem: self.problem.with(c),
        }
    }

    #[cfg(not(feature = "with-cplex"))]
    pub fn solve(
        self,
    ) -> Result<<CoinCbcProblem as SolverModel>::Solution, <CoinCbcProblem as SolverModel>::Error>
    {
        self.problem.solve()
    }

    #[cfg(feature = "with-cplex")]
    pub fn solve(
        self,
    ) -> Result<<CPLEXProblem as SolverModel>::Solution, <CPLEXProblem as SolverModel>::Error> {
        self.problem.solve()
    }
}

#[derive(Debug, Clone)]
struct PackingConfig {
    time_limit: Option<u32>, // in seconds
    size_x: u32,
    size_y: u32,
    min_z: i32,
    max_z: u32,
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
                    for z in 0..config.max_z {
                        let schedule = Schedule::new(x as i32, y as i32, z as i32, r, f == 1);
                        let scheduled = apply_schedule(program, &schedule);
                        let poly = scheduled.polycube().unwrap();
                        if poly.blocks().iter().all(|b| {
                            (b.x as u32) < config.size_x
                                && (b.y as u32) < config.size_y
                                && (b.z as u32) < config.max_z
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
/// * Constant values:
///   * X, Y                = the chip size (constant value)
///   * sx[i], sy[i], sz[i] = the size of i-th cuboid
///   * Z                   = sum_i sz[i]
///   * dx, dy, dz[i][j]    = relative positions of cuboids (if i, j are in the same program)
/// * Variables:
///   * a[i][j] := (x pos of i-th cuboid) > (x pos of j-th cuboid) (binary)
///   * b[i][j] := (y pos of i-th cuboid) > (y pos of j-th cuboid) (binary)
///   * c[i][j] := (z pos of i-th cuboid) > (z pos of j-th cuboid) (binary)
///   * x[i], y[i], z[i] := the position of i-th cuboid
///   * v := the objective value
///
/// Minimize v
/// s.t.
///   * a[i][j] + a[j][i] + b[i][j] + b[j][i] + c[i][j] + c[j][i] >= 1
///   * x[i] - x[j] + X * a[i][j] <= X - sx[i] for all i, j
///   * y[i] - y[j] + Y * b[i][j] <= Y - sy[i] for all i, j
///   * z[i] - z[j] + Z * c[i][j] <= Z - sz[i] for all i, j
///   * x[i] + sx[i] <= X
///   * y[i] + sy[i] <= Y
///   * z[i] + sz[i] <= Z
///   * x[i] + dx[i][j] = x[j] (for all i, j in the same program)
///   * y[i] + dy[i][j] = y[j] (for all i, j in the same program)
///   * z[i] + dz[i][j] = z[j] (for all i, j in the same program)
///   * z[i] + sz[i] <= v
#[warn(dead_code)]
struct CuboidPackingProblem {
    config: PackingConfig,
    fixed_cuboids: Vec<Cuboid>,
    programs: Vec<Vec<Cuboid>>,
    vars: ProblemVariables,
    to_cuboid_idx: Vec<Vec<usize>>, // (program_idx, cuboid_idx in the program) -> cuboid_idx
    x: Vec<Variable>,
    y: Vec<Variable>,
    z: Vec<Variable>,
    a: HashMap<(usize, usize), Variable>, // (cuboid_idx, cuboid_idx) -> variable
    b: HashMap<(usize, usize), Variable>,
    c: HashMap<(usize, usize), Variable>,
    v: Variable,
    cuboid_size: Vec<[usize; 3]>,
}

// TODO: Consider rotations
impl CuboidPackingProblem {
    pub fn new(
        config: PackingConfig,
        fixed_cuboids: Vec<Cuboid>,
        programs: Vec<Vec<Cuboid>>,
    ) -> Self {
        assert!(!programs.is_empty());
        assert!(fixed_cuboids
            .iter()
            .all(|c| { 0 <= c.x1() && 0 <= c.y1() && 0 <= c.z1() }));

        let mut vars = variables!();
        let num_programs = programs.len();
        let num_cuboids = programs.iter().map(|cs| cs.len()).sum::<usize>();

        let mut to_cuboid_idx = vec![Vec::new(); num_programs];
        {
            let mut cnt = 0;
            for i in 0..programs.len() {
                for _ in &programs[i] {
                    to_cuboid_idx[i].push(cnt);
                    cnt += 1;
                }
            }
        }

        let x = (0..num_cuboids)
            .map(|_| vars.add(variable().integer()))
            .collect();
        let y = (0..num_cuboids)
            .map(|_| vars.add(variable().integer()))
            .collect();
        let z = (0..num_cuboids)
            .map(|_| vars.add(variable().integer()))
            .collect();
        let mut a = HashMap::new();
        let mut b = HashMap::new();
        let mut c = HashMap::new();
        for i1 in 0..programs.len() {
            for j1 in 0..programs[i1].len() {
                let id1 = to_cuboid_idx[i1][j1];
                for i2 in 0..programs.len() {
                    if i1 == i2 {
                        continue;
                    }
                    for j2 in 0..programs[i2].len() {
                        let id2 = to_cuboid_idx[i2][j2];
                        a.insert((id1, id2), vars.add(variable().binary()));
                        b.insert((id1, id2), vars.add(variable().binary()));
                        c.insert((id1, id2), vars.add(variable().binary()));
                    }
                }

                for i2 in 0..fixed_cuboids.len() {
                    let id2 = num_cuboids + i2;
                    a.insert((id1, id2), vars.add(variable().binary()));
                    a.insert((id2, id1), vars.add(variable().binary()));
                    b.insert((id1, id2), vars.add(variable().binary()));
                    b.insert((id2, id1), vars.add(variable().binary()));
                    c.insert((id1, id2), vars.add(variable().binary()));
                    c.insert((id2, id1), vars.add(variable().binary()));
                }
            }
        }
        let v = vars.add_variable();

        let cuboid_size = programs
            .iter()
            .flat_map(|cs| cs.iter().map(|c| [c.size_x(), c.size_y(), c.size_z()]))
            .collect();

        Self {
            config,
            fixed_cuboids,
            programs,
            to_cuboid_idx,
            vars,
            x,
            y,
            z,
            a,
            b,
            c,
            v,
            cuboid_size,
        }
    }

    pub fn solve(self) -> Vec<Schedule> {
        let mut problem = LPSolverWrapper::new(self.vars.minimise(self.v), self.config.time_limit);

        let max_x = self.config.size_x as i32; // X
        let max_y = self.config.size_y as i32; // Y
        let max_z = self.config.max_z as i32; // Z

        let num_targets = self.x.len();

        for i1 in 0..self.programs.len() {
            for j1 in 0..self.programs[i1].len() {
                let id1 = self.to_cuboid_idx[i1][j1];
                let [size_xi, size_yi, size_zi] = self.cuboid_size[id1];
                let (size_xi, size_yi, size_zi) = (size_xi as i32, size_yi as i32, size_zi as i32);
                let xi = self.x[id1];
                let yi = self.y[id1];
                let zi = self.z[id1];
                if j1 > 0 {
                    let id3 = self.to_cuboid_idx[i1][0];
                    let dx = self.programs[i1][0].x1() - self.programs[i1][j1].x1();
                    let dy = self.programs[i1][0].y1() - self.programs[i1][j1].y1();
                    let dz = self.programs[i1][0].z1() - self.programs[i1][j1].z1();
                    problem = problem
                        .with(constraint!(self.x[id3] == xi + dx))
                        .with(constraint!(self.y[id3] == yi + dy))
                        .with(constraint!(self.z[id3] == zi + dz));
                }

                problem = problem
                    .with(constraint!(0 <= xi))
                    .with(constraint!(xi + size_xi <= max_x))
                    .with(constraint!(0 <= yi))
                    .with(constraint!(yi + size_yi <= max_y))
                    .with(constraint!(self.config.min_z <= zi))
                    .with(constraint!(zi + size_zi <= max_z))
                    .with(constraint!(zi + size_zi <= self.v));

                // for fixed_cuboids
                for i2 in 0..self.fixed_cuboids.len() {
                    let id2 = num_targets + i2;
                    let aij = self.a[&(id1, id2)];
                    let aji = self.a[&(id2, id1)];
                    let bij = self.b[&(id1, id2)];
                    let bji = self.b[&(id2, id1)];
                    let cij = self.c[&(id1, id2)];
                    let cji = self.c[&(id2, id1)];
                    let xj = self.fixed_cuboids[i2].x1();
                    let yj = self.fixed_cuboids[i2].y1();
                    let zj = self.fixed_cuboids[i2].z1();
                    let size_xj = self.fixed_cuboids[i2].size_x() as i32;
                    let size_yj = self.fixed_cuboids[i2].size_y() as i32;
                    let size_zj = self.fixed_cuboids[i2].size_z() as i32;
                    problem = problem.with(constraint!(aij + aji + bij + bji + cij + cji >= 1));
                    problem = problem
                        .with(constraint!(xi - xj + max_x * aij <= max_x - size_xi))
                        .with(constraint!(yi - yj + max_y * bij <= max_y - size_yi))
                        .with(constraint!(zi - zj + max_z * cij <= max_z - size_zi))
                        .with(constraint!(xj - xi + max_x * aji <= max_x - size_xj))
                        .with(constraint!(yj - yi + max_y * bji <= max_y - size_yj))
                        .with(constraint!(zj - zi + max_z * cji <= max_z - size_zj));
                }

                // Other target cuboids
                for i2 in 0..self.programs.len() {
                    if i1 == i2 {
                        continue;
                    }
                    for j2 in 0..self.programs[i2].len() {
                        let id2 = self.to_cuboid_idx[i2][j2];
                        let aij = self.a[&(id1, id2)];
                        let aji = self.a[&(id2, id1)];
                        let bij = self.b[&(id1, id2)];
                        let bji = self.b[&(id2, id1)];
                        let cij = self.c[&(id1, id2)];
                        let cji = self.c[&(id2, id1)];
                        let xj = self.x[id2];
                        let yj = self.y[id2];
                        let zj = self.z[id2];

                        if id1 < id2 {
                            problem =
                                problem.with(constraint!(aij + aji + bij + bji + cij + cji >= 1));
                        }
                        problem = problem
                            .with(constraint!(xi - xj + max_x * aij <= max_x - size_xi))
                            .with(constraint!(yi - yj + max_y * bij <= max_y - size_yi))
                            .with(constraint!(zi - zj + max_z * cij <= max_z - size_zi));
                    }
                }
            }
        }

        let solution = problem.solve().unwrap();
        (0..self.programs.len())
            .map(|i| {
                let id = self.to_cuboid_idx[i][0];
                let x = solution.value(self.x[id]).round() as i32;
                let y = solution.value(self.y[id]).round() as i32;
                let z = solution.value(self.z[id]).round() as i32;
                let x_orig = self.programs[i][0].x1();
                let y_orig = self.programs[i][0].y1();
                let z_orig = self.programs[i][0].z1();
                Schedule::new(x - x_orig, y - y_orig, z - z_orig, 0, false)
            })
            .collect()
    }
}

pub struct LPScheduler {
    job_list: VecDeque<Job>,
    config: SimulationConfig,
    schedule_cycles_sum: u64,
    scheduled_count: u64,
}

impl LPScheduler {
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            job_list: VecDeque::new(),
            config,
            schedule_cycles_sum: 0,
            scheduled_count: 0,
        }
    }
}

impl Scheduler for LPScheduler {
    fn add_job(&mut self, job: Job) {
        self.job_list.push_back(job);
    }

    fn run(&mut self, env: &Environment) -> Vec<(JobID, Schedule)> {
        if self.job_list.is_empty() {
            return Vec::new();
        }

        let jobs = self.take_jobs_by_batch_size();

        let zsum = jobs
            .iter()
            .map(|job| match job.program.format() {
                ProgramFormat::Polycube(p) => {
                    p.blocks().iter().map(|c| c.z).max().unwrap() as u32 + 1
                }
                ProgramFormat::Cuboid(cs) => cs.iter().map(|c| c.size_z() as u32).sum(),
            })
            .sum::<u32>();

        let schedule_point =
            (env.global_pc() + self.schedule_cycles_sum / u64::max(1, self.scheduled_count)) as i32;

        let max_z = if env.end_pc() < schedule_point as u64 {
            zsum
        } else {
            zsum + (env.end_pc() - schedule_point as u64) as u32
        };

        // FIXME: If the z value is too large, errors will occur in the solver (or inside the
        // wrapper library) due to floating-point precision. This is a workaround to prevent that.
        let shrink_ratio = max_z / 100_000 + 1;

        let max_z = max_z.div_ceil(shrink_ratio) + jobs.len() as u32;
        let schedule_point = (schedule_point + shrink_ratio as i32 - 1) / (shrink_ratio as i32);

        let pack_cfg = PackingConfig {
            time_limit: self.config.scheduler.time_limit,
            size_x: self.config.size_x,
            size_y: self.config.size_y,
            min_z: 0,
            max_z,
        };

        let problem = if jobs.iter().all(|job| job.program.is_polycube()) {
            let programs = jobs.iter().map(|job| job.program.clone()).collect();
            PackingProblem::Polycube(PolycubePackingProblem::new(pack_cfg, programs))
        } else if jobs.iter().all(|job| job.program.is_cuboid()) {
            let shrink_cuboid = |c: &Cuboid, ref_point: i32| {
                let sr = shrink_ratio as i32;
                let z2 = (c.z2() + sr - 1) / sr;
                let z1 = c.z1() / sr;
                let size_z = z2 - z1;
                let (z1, size_z) = if z1 < ref_point {
                    let size_z = size_z - (ref_point - z1);
                    (0, size_z)
                } else {
                    let z1 = i32::max(0, z1 - ref_point);
                    (z1, size_z)
                };
                Cuboid::new(
                    Coordinate::new(c.x1(), c.y1(), z1),
                    c.size_x(),
                    c.size_y(),
                    size_z as usize,
                )
            };

            let mut fixed_cuboids: Vec<_> = env
                .running_programs()
                .iter()
                .flat_map(|p| {
                    p.cuboid().unwrap().iter().filter_map(|c| {
                        let sr = shrink_ratio as i32;
                        let z2 = (c.z2() + sr - 1) / sr;
                        if schedule_point < z2 {
                            let c = shrink_cuboid(c, schedule_point);
                            Some(c)
                        } else {
                            None
                        }
                    })
                })
                .collect();

            for move_region in env.defrag_move_areas() {
                let sr = shrink_ratio as i32;
                let z2 = (move_region.z2() + sr - 1) / sr;
                if schedule_point < z2 {
                    let c = shrink_cuboid(move_region, schedule_point);
                    fixed_cuboids.push(c);
                }
            }

            let cuboids = jobs
                .iter()
                .map(|p| {
                    p.program
                        .cuboid()
                        .unwrap()
                        .iter()
                        .map(|c| shrink_cuboid(c, 0))
                        .collect()
                })
                .collect();

            PackingProblem::Cuboid(CuboidPackingProblem::new(pack_cfg, fixed_cuboids, cuboids))
        } else {
            panic!("unsupported");
        };

        let start = Instant::now();
        let schedules = problem.solve();
        let elapsed = start
            .elapsed()
            .as_micros()
            .div_ceil(self.config.micro_sec_per_cycle.into());
        self.schedule_cycles_sum += elapsed as u64;
        self.scheduled_count += 1;

        // The schedules calculated with an empty environment, it is necessary to shift their z
        // position by the maximum z point of issued programs.
        let schedules: Vec<_> = schedules
            .into_iter()
            .map(|s| {
                Schedule::new(
                    s.x,
                    s.y,
                    (s.z + schedule_point) * (shrink_ratio as i32),
                    s.rotate,
                    s.flip,
                )
            })
            .collect();

        jobs.into_iter().map(|job| job.id).zip(schedules).collect()
    }
}

impl LPScheduler {
    fn take_jobs_by_batch_size(&mut self) -> Vec<Job> {
        let take_len = if let Some(batch_size) = self.config.scheduler.batch_size {
            usize::min(self.job_list.len(), batch_size as usize)
        } else {
            self.job_list.len()
        };
        let mut taken_jobs = self.job_list.split_off(take_len);
        std::mem::swap(&mut taken_jobs, &mut self.job_list);
        taken_jobs.into()
    }
}

#[cfg(test)]
pub mod test {
    use crate::program::{Coordinate, Cuboid, Polycube, Program, ProgramFormat};
    use crate::scheduler::lp_scheduler::{CuboidPackingProblem, PackingConfig};
    use crate::scheduler::{apply_schedule, apply_schedule_to_cuboid};

    #[test]
    fn test_lp_polycube() {
        use crate::scheduler::lp_scheduler::PolycubePackingProblem;

        let config = PackingConfig {
            time_limit: Some(60),
            size_x: 4,
            size_y: 3,
            min_z: 0,
            max_z: 8,
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

        let problem = PolycubePackingProblem::new(config.clone(), programs.clone());
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
                assert!(0 <= pos.z && (pos.z as u32) < config.max_z);
                max_z = i32::max(max_z, pos.z);
            }
            for scheduled_j in scheduled.iter().skip(i + 1) {
                let poly2 = scheduled_j.polycube().unwrap();
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
        let cuboid_1x1x1 = Cuboid::new(Coordinate::new(0, 0, 0), 1, 1, 1);
        let cuboid_1x2x1 = Cuboid::new(Coordinate::new(0, 0, 0), 1, 2, 1);
        let cuboid_1x2x2 = Cuboid::new(Coordinate::new(0, 0, 0), 1, 2, 2);
        let programs = vec![
            vec![cuboid_1x2x2],
            vec![cuboid_1x1x1.clone()],
            vec![cuboid_1x2x1],
            vec![cuboid_1x1x1],
        ];

        let config = PackingConfig {
            time_limit: Some(60),
            size_x: 2,
            size_y: 2,
            min_z: 0,
            max_z: 2,
        };

        let problem = CuboidPackingProblem::new(config.clone(), Vec::new(), programs.clone());
        let schedule = problem.solve();
        for i in 0..programs.len() {
            let xi = schedule[i].x;
            let yi = schedule[i].y;
            let zi = schedule[i].z;
            let size_xi = programs[i][0].size_x() as i32;
            let size_yi = programs[i][0].size_y() as i32;
            let size_zi = programs[i][0].size_z() as i32;
            assert!(0 <= xi && xi + size_xi <= config.size_x as i32);
            assert!(0 <= yi && yi + size_yi <= config.size_y as i32);
            assert!(0 <= zi && zi + size_zi <= config.max_z as i32);
            for j in i + 1..programs.len() {
                let xj = schedule[j].x;
                let yj = schedule[j].y;
                let zj = schedule[j].z;
                let size_xj = programs[j][0].size_x() as i32;
                let size_yj = programs[j][0].size_y() as i32;
                let size_zj = programs[j][0].size_z() as i32;
                let is_overlap_x = !(xi + size_xi <= xj || xj + size_xj <= xi);
                let is_overlap_y = !(yi + size_yi <= yj || yj + size_yj <= yi);
                let is_overlap_z = !(zi + size_zi <= zj || zj + size_zj <= zi);
                assert!(!is_overlap_x || !is_overlap_y || !is_overlap_z);
            }
        }
    }

    #[test]
    fn test_lp_k_cuboid() {
        let programs = vec![
            vec![
                Cuboid::new(Coordinate::new(0, 0, 0), 1, 1, 1),
                Cuboid::new(Coordinate::new(1, 0, 1), 1, 1, 1),
            ],
            vec![
                Cuboid::new(Coordinate::new(0, 0, 1), 1, 1, 1),
                Cuboid::new(Coordinate::new(1, 0, 0), 1, 1, 1),
            ],
        ];

        let config = PackingConfig {
            time_limit: Some(60),
            size_x: 2,
            size_y: 1,
            max_z: 2,
            min_z: 0,
        };

        let problem = CuboidPackingProblem::new(config.clone(), Vec::new(), programs.clone());
        let schedule = problem.solve();
        let results: Vec<_> = programs
            .into_iter()
            .enumerate()
            .flat_map(|(i, cs)| {
                let schedule = &schedule[i];
                cs.into_iter()
                    .map(|c| apply_schedule_to_cuboid(&c, schedule))
            })
            .collect();

        for i in 0..results.len() {
            let xi = results[i].pos().x;
            let yi = results[i].pos().y;
            let zi = results[i].pos().z;
            let size_xi = results[i].size_x() as i32;
            let size_yi = results[i].size_y() as i32;
            let size_zi = results[i].size_z() as i32;
            assert!(0 <= xi && xi + size_xi <= config.size_x as i32);
            assert!(0 <= yi && yi + size_yi <= config.size_y as i32);
            assert!(0 <= zi && zi + size_zi <= config.max_z as i32);
            for result_j in results.iter().skip(i + 1) {
                let xj = result_j.pos().x;
                let yj = result_j.pos().y;
                let zj = result_j.pos().z;
                let size_xj = result_j.size_x() as i32;
                let size_yj = result_j.size_y() as i32;
                let size_zj = result_j.size_z() as i32;
                let is_overlap_x = !(xi + size_xi <= xj || xj + size_xj <= xi);
                let is_overlap_y = !(yi + size_yi <= yj || yj + size_yj <= yi);
                let is_overlap_z = !(zi + size_zi <= zj || zj + size_zj <= zi);
                assert!(!is_overlap_x || !is_overlap_y || !is_overlap_z);
            }
        }
    }
}
