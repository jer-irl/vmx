use std::collections::HashMap;
use std::cell::RefCell;

const RP_IDX: RegIdx = RegIdx(15);

pub struct ProgramInstance {
    program: Program,
    state: ExecutionState,
}

impl ProgramInstance {
    pub fn new(program: Program) -> Self {
        Self { program, state: ExecutionState::default() }
    }

    pub fn execute_step(&mut self) -> Result<bool, ()> {
        let instruction = self.program.instructions.get(self.state.register_read(RP_IDX) as usize).ok_or(())?;
        match instruction {
            Instruction::ArrIns { arr, idx, val } => {
                self.state.array_insert(
                    self.state.register_read(*arr) as u64, 
                    self.state.register_read(*idx) as u64, 
                    self.state.register_read(*val)
                );
                self.state.incremement_rp();
                return Ok(true)
            },
            Instruction::ArrGet { dst, arr, idx } => {
                self.state.register_write(*dst, self.state.array_read(self.state.register_read(*arr) as u64, self.state.register_read(*idx) as u64));
                self.state.incremement_rp();
                return Ok(true)
            },
            Instruction::MovImm { dst, imm } => {
                self.state.register_write(*dst, *imm as i64);
                self.state.incremement_rp();
                return Ok(true)
            },
            Instruction::Mov { dst, src } => {
                self.state.register_write(*dst, self.state.register_read(*src));
                self.state.incremement_rp();
                return Ok(true)
            },
            Instruction::Jmp { adr } => {
                self.state.register_write(RP_IDX, self.state.register_read(*adr));
                return Ok(true)
            },
            Instruction::Jeq { adr, v0, v1 } => {
                if self.state.register_read(*v0) == self.state.register_read(*v1) {
                    self.state.register_write(RP_IDX, self.state.register_read(*adr));
                } else {
                    self.state.incremement_rp();
                }
                return Ok(true)
            },
            Instruction::Jne { adr, v0, v1 } => {
                if self.state.register_read(*v0) != self.state.register_read(*v1) {
                    self.state.register_write(RP_IDX, self.state.register_read(*adr));
                } else {
                    self.state.incremement_rp();
                }
                return Ok(true)
            },
            Instruction::Jgt { adr, v0, v1 } => {
                if self.state.register_read(*v0) > self.state.register_read(*v1) {
                    self.state.register_write(RP_IDX, self.state.register_read(*adr));
                } else {
                    self.state.incremement_rp();
                }
                return Ok(true)
            },
            Instruction::Jge { adr, v0, v1 } => {
                if self.state.register_read(*v0) >= self.state.register_read(*v1) {
                    self.state.register_write(RP_IDX, self.state.register_read(*adr));
                } else {
                    self.state.incremement_rp();
                }
                return Ok(true)
            },
            Instruction::Jlt { adr, v0, v1 } => {
                if self.state.register_read(*v0) < self.state.register_read(*v1) {
                    self.state.register_write(RP_IDX, self.state.register_read(*adr));
                } else {
                    self.state.incremement_rp();
                }
                return Ok(true)
            },
            Instruction::Jle { adr, v0, v1 } => {
                if self.state.register_read(*v0) <= self.state.register_read(*v1) {
                    self.state.register_write(RP_IDX, self.state.register_read(*adr));
                } else {
                    self.state.incremement_rp();
                }
                return Ok(true)
            },
            Instruction::Add { dst, v0, v1 } => {
                let val = self.state.register_read(*v0) + self.state.register_read(*v1);
                self.state.register_write(*dst, val);
                return Ok(true)
            },
            Instruction::Mul { dst, v0, v1 } => {
                let val = self.state.register_read(*v0) * self.state.register_read(*v1);
                self.state.register_write(*dst, val);
                return Ok(true)
            },
            Instruction::Div { dst, v0, v1 } => {
                let val = self.state.register_read(*v0) / self.state.register_read(*v1);
                self.state.register_write(*dst, val);
                return Ok(true)
            },
            Instruction::Mod { dst, v0, v1 } => {
                let val = self.state.register_read(*v0) % self.state.register_read(*v1);
                self.state.register_write(*dst, val);
                return Ok(true)
            },
            Instruction::Halt { } => return Ok(false),
        }
    }
}

pub struct Program {
    instructions: Vec<Instruction>,
}

#[derive(Default)]
struct Array(HashMap<u64, i64>);

impl Array {
    pub fn insert(&mut self, idx: u64, val: i64) {
        self.0.insert(idx, val);
    }
    pub fn get(&self, idx: u64) -> i64 {
        *self.0.get(&idx).unwrap_or(&0)
    }
}

#[derive(Clone, Copy)]
struct Register(i64);

pub struct ExecutionState {
    arrays: RefCell<HashMap<u64, Array>>,
    registers: [Register; 16],
}

impl ExecutionState {
    pub fn default() -> Self {
        Self {
            arrays: RefCell::default(),
            registers: [Register(0); 16],
        }
    }

    pub fn array_insert(&mut self, arr: u64, idx: u64, val: i64) {
        self.arrays
            .borrow_mut()
            .entry(arr)
            .or_default()
            .insert(idx, val)
    }

    pub fn array_read(&self, arr: u64, idx: u64) -> i64 {
        self.arrays
            .borrow_mut()
            .entry(arr)
            .or_default()
            .get(idx)
    }

    pub fn register_write(&mut self, idx: RegIdx, val: i64) {
        self.registers[idx.0 as usize].0 = val
    }

    pub fn register_read(&self, idx: RegIdx) -> i64 {
        self.registers[idx.0 as usize].0
    }

    pub fn incremement_rp(&mut self) {
        self.registers[RP_IDX.0 as usize].0 += 1;
    }
}

#[derive(Copy, Clone)]
pub struct RegIdx(u8);

impl From<u8> for RegIdx {
    fn from(n: u8) -> Self {
        Self(n)
    }
}

pub enum Instruction {
    ArrIns {
        val: RegIdx,
        arr: RegIdx,
        idx: RegIdx,
    },
    ArrGet {
        dst: RegIdx,
        arr: RegIdx,
        idx: RegIdx,
    },
    MovImm {
        dst: RegIdx,
        imm: i32,
    },
    Mov {
        dst: RegIdx,
        src: RegIdx,
    },
    Jmp {
        adr: RegIdx,
    },
    Jeq {
        adr: RegIdx,
        v0: RegIdx,
        v1: RegIdx,
    },
    Jne {
        adr: RegIdx,
        v0: RegIdx,
        v1: RegIdx,
    },
    Jgt {
        adr: RegIdx,
        v0: RegIdx,
        v1: RegIdx,
    },
    Jge {
        adr: RegIdx,
        v0: RegIdx,
        v1: RegIdx,
    },
    Jlt {
        adr: RegIdx,
        v0: RegIdx,
        v1: RegIdx,
    },
    Jle {
        adr: RegIdx,
        v0: RegIdx,
        v1: RegIdx,
    },
    Add {
        dst: RegIdx,
        v0: RegIdx,
        v1: RegIdx,
    },
    Mul {
        dst: RegIdx,
        v0: RegIdx,
        v1: RegIdx,
    },
    Div {
        dst: RegIdx,
        v0: RegIdx,
        v1: RegIdx,
    },
    Mod {
        dst: RegIdx,
        v0: RegIdx,
        v1: RegIdx,
    },
    Halt {},
}
