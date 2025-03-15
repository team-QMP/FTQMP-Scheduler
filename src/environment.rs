use crate::program::{
    cut_program_at_z, is_overlap, Cuboid, Program, ProgramCounter, ProgramFormat,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Environment {
    issued_programs: Vec<Program>,
    size_x: i32,
    size_y: i32,
    /// The maximum z position of issued programs + 1.
    end_pc: u64,
    /// The global time in cycles. This is not equal to the program counter because the program counter may
    /// suspended during execution.
    current_time: u64,
    /// The global program counter (i.e., $z$ position) in cycles.
    /// TODO: add program counter to each program (or job)
    program_counter: u64,
    suspend_until: BTreeMap<u64, ProgramCounter>,
}

impl Environment {
    pub fn new(size_x: i32, size_y: i32) -> Self {
        Self {
            issued_programs: Vec::new(),
            size_x,
            size_y,
            end_pc: 0,
            current_time: 0,
            program_counter: 0,
            suspend_until: BTreeMap::new(),
        }
    }

    pub fn can_issue(&self, p: &Program) -> bool {
        let is_in_range = match p.format() {
            ProgramFormat::Polycube(polycube) => polycube.blocks().iter().all(|b| {
                0 <= b.x && b.x < self.size_x && 0 <= b.y && b.y < self.size_y && 0 <= b.z
            }),
            ProgramFormat::Cuboid(cs) => cs.iter().all(|c| {
                let pos = c.pos();
                0 <= pos.x
                    && pos.x + (c.size_x() as i32) <= self.size_x
                    && 0 <= pos.y
                    && pos.y + (c.size_y() as i32) <= self.size_y
                    && 0 <= pos.z
            }),
        };
        let is_overlap = self.issued_programs.iter().any(|p2| is_overlap(p, p2));
        is_in_range && !is_overlap
    }

    pub fn issue_program(&mut self, p: &Program) -> bool {
        let can_issue = self.can_issue(p);
        if can_issue {
            self.issued_programs.push(p.clone());
            match p.format() {
                ProgramFormat::Polycube(p) => {
                    for b in p.blocks() {
                        self.end_pc = u64::max(self.end_pc, b.z as u64 + 1);
                    }
                }
                ProgramFormat::Cuboid(cs) => {
                    for c in cs {
                        self.end_pc =
                            u64::max(self.end_pc, c.pos().z as u64 + c.size_z() as u64 + 1);
                    }
                }
            }
        }
        can_issue
    }

    pub fn issued_programs(&self) -> &Vec<Program> {
        &self.issued_programs
    }

    pub fn end_pc(&self) -> u64 {
        self.end_pc
    }

    pub fn current_time(&self) -> u64 {
        self.current_time
    }

    pub fn remaining_cycles(&self) -> u64 {
        let mut result = 0;
        let mut tmp_current_time = self.current_time;
        let mut tmp_program_counter = self.program_counter;
        for (suspend_point, until) in &self.suspend_until {
            if *suspend_point >= self.end_pc {
                break;
            }
            assert!(*suspend_point >= tmp_program_counter);
            let tmp_advance_cycles = *suspend_point - tmp_program_counter;
            result += tmp_advance_cycles;
            tmp_current_time += tmp_advance_cycles;
            tmp_program_counter = *suspend_point;
            let wait_cycles = if *until > tmp_current_time {
                until - tmp_current_time
            } else {
                0
            };
            result += wait_cycles;
            tmp_current_time += wait_cycles;
        }
        result += self.end_pc() - tmp_program_counter;

        result
    }

    /// Returns the global program counter
    pub fn global_pc(&self) -> u64 {
        self.program_counter
    }

    pub fn suspend_at(&mut self, suspend_point: ProgramCounter, until: u64) {
        assert!(self.program_counter <= suspend_point);
        if let Some(c) = self.suspend_until.get_mut(&suspend_point) {
            *c = (*c).max(until);
        } else {
            self.suspend_until.insert(suspend_point, until);
        }
    }

    pub fn advance_by(&mut self, mut advance_cycles: u64) {
        // TODO: refactoring?
        while let Some((suspend_point, until)) = self.suspend_until.pop_first() {
            if self.program_counter + advance_cycles > suspend_point {
                // proceed to the suspend point
                let tmp_advance_cycles = self.program_counter + advance_cycles - suspend_point;
                self.current_time += tmp_advance_cycles;
                self.program_counter = suspend_point;
                advance_cycles -= tmp_advance_cycles;
                // then wait
                let wait_cycles = if until > self.current_time {
                    advance_cycles.min(until - self.current_time)
                } else {
                    0
                };
                self.current_time += wait_cycles;
                advance_cycles -= wait_cycles;
            } else {
                // leave the entry
                self.suspend_until.insert(suspend_point, until);
                break;
            }
        }

        self.current_time += advance_cycles;
        self.program_counter += advance_cycles;
    }

    // Perform defragmentation in the current program counter
    pub fn defrag(&mut self) {
        // TODO: more efficient implementation?
        let (below, above) = self.issued_programs.iter().fold(
            (Vec::new(), Vec::new()),
            |(mut below, mut above), program| {
                let (below_c, above_c) =
                    cut_program_at_z(program.clone(), self.program_counter as i32);
                if let Some(below_c) = below_c {
                    below.push(below_c);
                }
                if let Some(above_c) = above_c {
                    above.push(above_c);
                }
                (below, above)
            },
        );
        self.issued_programs.clear();
        self.issued_programs.extend(below);
        self.issued_programs.extend(drop_programs(above));

        let cost_of_defrag = (self.size_x + self.size_y) as ProgramCounter; // worst scenario
        self.suspend_at(self.program_counter, self.current_time + cost_of_defrag);
    }
}

/// All program must be represented by a single cuboid.
/// TODO: Support other representations?
fn drop_programs(programs: Vec<Program>) -> Vec<Program> {
    let mut cuboids: Vec<_> = programs
        .into_iter()
        .map(|p| {
            assert!(p.cuboid().map_or(false, |c| c.len() == 1));
            p.cuboid().unwrap()[0].clone()
        })
        .collect();

    // drop by y position
    cuboids.sort_by_key(|c| c.y1());
    let mut cs_drop_x: Vec<Cuboid> = Vec::new();
    for mut c in cuboids {
        let mut new_y1 = 0;
        for other in &cs_drop_x {
            if !(c.x2() <= other.x1()
                || c.x1() >= other.x2()
                || c.z2() <= other.z1()
                || c.z1() >= other.z2())
            {
                new_y1 = new_y1.max(other.y2());
            }
        }
        c.update_y1(new_y1);
        cs_drop_x.push(c);
    }

    // drop by x position
    cs_drop_x.sort_by_key(|c| c.x1());
    let mut result: Vec<Cuboid> = Vec::new();
    for mut c in cs_drop_x {
        let mut new_x1 = 0;
        for other in &result {
            if !(c.y2() <= other.y1()
                || c.y1() >= other.y2()
                || c.z2() <= other.z1()
                || c.z1() >= other.z2())
            {
                new_x1 = new_x1.max(other.x2());
            }
        }
        c.update_x1(new_x1);
        result.push(c.clone());
    }

    result
        .into_iter()
        .map(|c| Program::new(ProgramFormat::Cuboid(vec![c])))
        .collect()
}

#[cfg(test)]
mod test {
    use crate::environment::Environment;
    use crate::program::{Coordinate, Cuboid, Polycube, Program, ProgramFormat};

    use super::drop_programs;

    #[test]
    fn test_environment_add_polycube() {
        let mut env = Environment::new(100, 100);

        let p1 = Program::new(ProgramFormat::Polycube(Polycube::new(vec![
            Coordinate::new(0, 0, 0),
        ])));
        let p2 = Program::new(ProgramFormat::Polycube(Polycube::new(vec![
            Coordinate::new(1, 1, 1),
        ])));
        let p3 = Program::new(ProgramFormat::Polycube(Polycube::new(vec![
            Coordinate::new(0, 0, 0),
        ])));

        assert!(env.issue_program(&p1));
        assert!(env.issue_program(&p2));
        assert!(!env.issue_program(&p3));

        let expected = vec![p1, p2];
        assert_eq!(env.issued_programs(), &expected);
    }

    #[test]
    fn test_drop_programs() {
        let c1 = Cuboid::new(Coordinate::new(0, 1, 0), 2, 2, 2);
        let c2 = Cuboid::new(Coordinate::new(2, 0, 0), 2, 2, 2);
        let c3 = Cuboid::new(Coordinate::new(1, 3, 1), 2, 2, 2);
        let p1 = Program::new(ProgramFormat::Cuboid(vec![c1]));
        let p2 = Program::new(ProgramFormat::Cuboid(vec![c2]));
        let p3 = Program::new(ProgramFormat::Cuboid(vec![c3]));

        let ps = drop_programs(vec![p1, p2, p3]);

        let c1_moved = Cuboid::new(Coordinate::new(0, 0, 0), 2, 2, 2);
        let c2_moved = Cuboid::new(Coordinate::new(2, 0, 0), 2, 2, 2);
        let c3_moved = Cuboid::new(Coordinate::new(0, 2, 1), 2, 2, 2);
        let p1_moved = Program::new(ProgramFormat::Cuboid(vec![c1_moved]));
        let p2_moved = Program::new(ProgramFormat::Cuboid(vec![c2_moved]));
        let p3_moved = Program::new(ProgramFormat::Cuboid(vec![c3_moved]));
        eprintln!("{:?}", ps);

        assert!(ps.contains(&p1_moved));
        assert!(ps.contains(&p2_moved));
        assert!(ps.contains(&p3_moved));
    }
}
