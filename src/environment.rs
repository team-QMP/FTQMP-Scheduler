use crate::program::{is_overlap, Program, ProgramCounter, ProgramFormat};
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

    pub fn remaining_cycles(&self) -> u64 {
        let mut result = 0;
        let mut tmp_current_time = self.current_time;
        let mut tmp_program_counter = self.program_counter;
        for (suspend_point, until) in &self.suspend_until {
            assert!(*suspend_point >= tmp_program_counter);
            let tmp_advance_cycles = suspend_point - tmp_program_counter;
            result += tmp_advance_cycles;
            tmp_current_time += tmp_advance_cycles;
            tmp_program_counter = *suspend_point;
            let wait_cycles = if *until > tmp_current_time {
                until - tmp_current_time
            } else {
                0
            };
            result += wait_cycles;
            tmp_current_time = *until;
        }
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
        loop {
            if let Some((suspend_point, until)) = self.suspend_until.pop_first() {
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
                    advance_cycles -= wait_cycles;
                } else {
                    // leave the entry
                    self.suspend_until.insert(suspend_point, until);
                    break;
                }
            } else {
                self.current_time += advance_cycles;
                self.program_counter += advance_cycles;
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::environment::Environment;
    use crate::program::{Coordinate, Polycube, Program, ProgramFormat};

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
}
