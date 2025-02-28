use crate::config::SimulationConfig;
use crate::environment::Environment;
use crate::job::Job;
use crate::program::{is_overlap, Coordinate, Program, ProgramFormat};
use crate::scheduler::{apply_schedule, JobID, Schedule, Scheduler};

use std::collections::VecDeque;

pub struct FastGreedyScheduler {
    job_list: VecDeque<Job>,
    config: SimulationConfig,
    location_candidates: Vec<Coordinate>,
}

impl FastGreedyScheduler {
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            job_list: VecDeque::new(),
            config,
            location_candidates: vec![Coordinate::new(0, 0, 0)],
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

impl Scheduler for FastGreedyScheduler {
    fn add_job(&mut self, job: Job) {
        self.job_list.push_back(job);
    }

    fn run(&mut self, env: &Environment) -> Vec<(JobID, Schedule)> {
        // TODO: refactoring
        self.location_candidates = self
            .location_candidates
            .iter()
            .filter_map(|candidate| {
                if candidate.z as u64 >= env.program_counter() {
                    Some(candidate.clone())
                } else {
                    None
                }
            })
            .collect();

        let mut res = Vec::new();
        let mut scheduled_programs= Vec::new(); // programs to be issued in this scheduling
        let jobs = self.take_jobs_by_batch_size();
        for job in jobs {
            let mut best_it = None;
            let mut best: Option<Schedule> = None;
            for (i, candidate) in self.location_candidates.iter().enumerate() {
                for rot in 0..4 {
                    for flip in [false, true] {
                        let schedule =
                            Schedule::new(candidate.x, candidate.y, candidate.z, rot, flip);
                        let scheduled_program = apply_schedule(&job.program, &schedule);
                        let is_overlap = scheduled_programs.iter()
                            .any(|p| is_overlap(&scheduled_program, p));
                        if !is_overlap && env.can_issue(&scheduled_program) {
                            if best.is_none() || best.clone().unwrap().z > schedule.z {
                                best = Some(schedule);
                                best_it = Some(i);
                            }
                        }
                    }
                }
            }

            let best_schedule = best.unwrap();
            let scheduled_program = apply_schedule(&job.program, &best_schedule);
            self.location_candidates.remove(best_it.expect(""));
            self.location_candidates
                .extend(create_location_candidate(&scheduled_program));
            scheduled_programs.push(scheduled_program);
            res.push((job.id, best_schedule));
        }

        res
    }
}
