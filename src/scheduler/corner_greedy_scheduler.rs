use crate::config::SimulationConfig;
use crate::environment::Environment;
use crate::job::Job;
use crate::program::{is_overlap, Coordinate, Program, ProgramFormat};
use crate::scheduler::{apply_schedule, JobID, Schedule, Scheduler};

use std::collections::{HashSet, VecDeque};
use std::time::Instant;

pub struct CornerGreedyScheduler {
    job_list: VecDeque<Job>,
    config: SimulationConfig,
    schedule_cycles_sum: u64,
    schedule_count: u64,
}

impl CornerGreedyScheduler {
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            job_list: VecDeque::new(),
            config,
            schedule_cycles_sum: 0,
            schedule_count: 0,
        }
    }

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

fn create_location_candidate(p: &Program) -> Vec<Coordinate> {
    let (x1, x2, y1, y2, z1, z2) = match p.format() {
        ProgramFormat::Polycube(p) => (
            p.min_x(),
            p.max_x(),
            p.min_y(),
            p.max_y(),
            p.min_z(),
            p.max_z(),
        ),
        ProgramFormat::Cuboid(cs) => cs.iter().fold(
            (i32::MAX, i32::MIN, i32::MAX, i32::MIN, i32::MAX, i32::MIN),
            |(x1, x2, y1, y2, z1, z2), c| {
                (
                    i32::min(x1, c.pos().x),
                    i32::max(x2, c.pos().x + (c.size_x() as i32)),
                    i32::min(y1, c.pos().y),
                    i32::max(y2, c.pos().y + (c.size_y() as i32)),
                    i32::min(z1, c.pos().z),
                    i32::max(z2, c.pos().z + (c.size_z() as i32)),
                )
            },
        ),
    };

    vec![
        Coordinate::new(x2, y1, z1),
        Coordinate::new(x1, y2, z1),
        Coordinate::new(x1, y1, z2),
        Coordinate::new(0, 0, z2),
    ]
}

impl Scheduler for CornerGreedyScheduler {
    fn add_job(&mut self, job: Job) {
        self.job_list.push_back(job);
    }

    fn run(&mut self, env: &Environment) -> Vec<(JobID, Schedule)> {
        let est_scheduling_cost = if self.schedule_count == 0 {
            0
        } else {
            self.schedule_cycles_sum / self.schedule_count
        };

        let start = Instant::now();

        let scheduled_point = env.global_pc() + est_scheduling_cost;

        let already_used: HashSet<_> = env.running_programs().iter().map(|p| p.pos()).collect();

        // TODO: incremental management of location candidates
        let mut location_candidates: Vec<_> = env
            .running_programs()
            .iter()
            .filter(|prog| prog.z2() as u64 > scheduled_point)
            .flat_map(|prog| {
                let cuboids = prog.cuboid().unwrap();
                assert!(cuboids.len() == 1);
                let x1 = cuboids[0].x1();
                let x2 = cuboids[0].x2();
                let y1 = cuboids[0].y1();
                let y2 = cuboids[0].y2();
                let z1 = cuboids[0].z1().max(scheduled_point as i32);
                let z2 = cuboids[0].z2();
                vec![
                    Coordinate::new(x2, y1, z1),
                    Coordinate::new(x1, y2, z1),
                    Coordinate::new(x1, y1, z2),
                    Coordinate::new(0, 0, z2),
                ]
                .into_iter()
                .filter(|pos| !already_used.contains(pos))
                .collect::<Vec<_>>()
            })
            .collect();
        if location_candidates.is_empty() {
            location_candidates.push(Coordinate::new(0, 0, scheduled_point as i32));
        }

        let defrag_move_areas = env.defrag_move_areas();

        tracing::debug!(
            "PC = {},  #(location candidates) = {},  #(defrag_move_areas) = {}",
            env.global_pc(),
            location_candidates.len(),
            defrag_move_areas.len()
        );

        let mut res = Vec::new();
        let mut scheduled_programs = Vec::new(); // programs to be issued in this scheduling
        let jobs = self.take_jobs_by_batch_size();
        let cmp_schedule = |s1: &Schedule, s2: &Schedule| (s1.z, s1.x + s1.y) < (s2.z, s2.x + s2.y);
        for job in jobs {
            let mut best_it = None;
            let mut best: Option<Schedule> = None;
            for (i, candidate) in location_candidates.iter().enumerate() {
                for rot in 0..4 {
                    let schedule = Schedule::new(candidate.x, candidate.y, candidate.z, rot, false);
                    let scheduled_program = apply_schedule(&job.program, &schedule);
                    let is_overlap = scheduled_programs
                        .iter()
                        .any(|p| is_overlap(&scheduled_program, p));
                    let is_overlap_with_moves = defrag_move_areas.iter().any(|c1| {
                        assert!(c1.z1() == c1.z2()); // because c1 is dummy cuboid
                        let c2 = &scheduled_program.cuboid().unwrap()[0];
                        let is_overlap_x = !(c1.x2() <= c2.x1() || c2.x2() <= c1.x1());
                        let is_overlap_y = !(c1.y2() <= c2.y1() || c2.y2() <= c1.y1());
                        let is_overlap_z = c2.z1() < c1.z1() && c1.z1() < c2.z2();
                        is_overlap_x && is_overlap_y && is_overlap_z
                    });
                    if !is_overlap
                        && !is_overlap_with_moves
                        && env.can_issue(&scheduled_program)
                        && (best.is_none() || cmp_schedule(&schedule, best.as_ref().unwrap()))
                    {
                        best = Some(schedule);
                        best_it = Some(i);
                    }
                }
            }

            let best_schedule = best.unwrap();
            let scheduled_program = apply_schedule(&job.program, &best_schedule);
            location_candidates.remove(best_it.expect(""));
            location_candidates.extend(create_location_candidate(&scheduled_program));
            scheduled_programs.push(scheduled_program);
            res.push((job.id, best_schedule));
        }

        let elapsed = start
            .elapsed()
            .as_micros()
            .div_ceil(self.config.micro_sec_per_cycle.into()) as u64;
        self.schedule_cycles_sum += elapsed;
        self.schedule_count += 1;

        res
    }
}
