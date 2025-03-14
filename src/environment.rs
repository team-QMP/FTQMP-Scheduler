use crate::program::{is_overlap, Program, ProgramFormat};

#[derive(Debug, Clone)]
pub struct Environment {
    issued_programs: Vec<Program>,
    size_x: i32,
    size_y: i32,
    /// The maximum z position of issued programs + 1.
    end_pc: u64,
    /// The global program counter (i.e., $z$ position)
    /// TODO: add program counter to each program (or job)
    program_counter: u64,
    suspend_cycle: u64,
}

impl Environment {
    pub fn new(size_x: i32, size_y: i32) -> Self {
        Self {
            issued_programs: Vec::new(),
            size_x,
            size_y,
            end_pc: 0,
            program_counter: 0,
            suspend_cycle: 0,
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
        self.end_pc - self.program_counter + self.suspend_cycle
    }

    /// Returns the global program counter
    pub fn global_pc(&self) -> u64 {
        self.program_counter
    }

    pub fn add_suspend_cycle(&mut self, count: u64) {
        self.suspend_cycle += count;
    }

    pub fn proceed_cycle(&mut self, count: u64) {
        let consumed_suspend_cycle = self.suspend_cycle.min(count);
        self.program_counter += count - consumed_suspend_cycle;
        self.suspend_cycle -= consumed_suspend_cycle;
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
