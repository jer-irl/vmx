use std::cell::RefCell;
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;

const RP_IDX: RegIdx = RegIdx(15);

pub struct ProgramInstance {
    program: Program,
    state: ExecutionState,
}

impl ProgramInstance {
    pub fn new(program: Program, state: ExecutionState) -> Self {
        Self { program, state }
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
                Ok(true)
            }
            Instruction::ArrGet { dst, arr, idx } => {
                self.state.register_write(*dst, self.state.array_read(self.state.register_read(*arr) as u64, self.state.register_read(*idx) as u64));
                self.state.incremement_rp();
                Ok(true)
            }
            Instruction::MovImm { dst, imm } => {
                self.state.register_write(*dst, *imm as i64);
                self.state.incremement_rp();
                Ok(true)
            }
            Instruction::Mov { dst, src } => {
                self.state.register_write(*dst, self.state.register_read(*src));
                self.state.incremement_rp();
                Ok(true)
            }
            Instruction::Jmp { adr } => {
                self.state.register_write(RP_IDX, self.state.register_read(*adr));
                Ok(true)
            }
            Instruction::Jeq { adr, v0, v1 } => {
                if self.state.register_read(*v0) == self.state.register_read(*v1) {
                    self.state.register_write(RP_IDX, self.state.register_read(*adr));
                } else {
                    self.state.incremement_rp();
                }
                Ok(true)
            }
            Instruction::Jne { adr, v0, v1 } => {
                if self.state.register_read(*v0) != self.state.register_read(*v1) {
                    self.state.register_write(RP_IDX, self.state.register_read(*adr));
                } else {
                    self.state.incremement_rp();
                }
                Ok(true)
            }
            Instruction::Jgt { adr, v0, v1 } => {
                if self.state.register_read(*v0) > self.state.register_read(*v1) {
                    self.state.register_write(RP_IDX, self.state.register_read(*adr));
                } else {
                    self.state.incremement_rp();
                }
                Ok(true)
            }
            Instruction::Jge { adr, v0, v1 } => {
                if self.state.register_read(*v0) >= self.state.register_read(*v1) {
                    self.state.register_write(RP_IDX, self.state.register_read(*adr));
                } else {
                    self.state.incremement_rp();
                }
                Ok(true)
            }
            Instruction::Jlt { adr, v0, v1 } => {
                if self.state.register_read(*v0) < self.state.register_read(*v1) {
                    self.state.register_write(RP_IDX, self.state.register_read(*adr));
                } else {
                    self.state.incremement_rp();
                }
                Ok(true)
            }
            Instruction::Jle { adr, v0, v1 } => {
                if self.state.register_read(*v0) <= self.state.register_read(*v1) {
                    self.state.register_write(RP_IDX, self.state.register_read(*adr));
                } else {
                    self.state.incremement_rp();
                }
                Ok(true)
            }
            Instruction::Add { dst, v0, v1 } => {
                let val = self.state.register_read(*v0) + self.state.register_read(*v1);
                self.state.register_write(*dst, val);
                self.state.incremement_rp();
                Ok(true)
            }
            Instruction::Mul { dst, v0, v1 } => {
                let val = self.state.register_read(*v0) * self.state.register_read(*v1);
                self.state.register_write(*dst, val);
                self.state.incremement_rp();
                Ok(true)
            }
            Instruction::Div { dst, v0, v1 } => {
                let val = self.state.register_read(*v0) / self.state.register_read(*v1);
                self.state.register_write(*dst, val);
                self.state.incremement_rp();
                Ok(true)
            }
            Instruction::Mod { dst, v0, v1 } => {
                let val = self.state.register_read(*v0) % self.state.register_read(*v1);
                self.state.register_write(*dst, val);
                self.state.incremement_rp();
                Ok(true)
            }
            Instruction::Halt { } => Ok(false),
            Instruction::Noop { } => {
                self.state.incremement_rp();
                Ok(true)
            }
        }
    }
}

pub struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    pub fn try_from_str(s: &str) -> Result<Self, ()> {
        let instructions: Vec<_> = s
            .split_whitespace()
            .map(|line| Instruction::try_from_line(line).expect("TODO").expect("TODO"))
            .collect();
        Ok(Self { instructions })
    }

    pub fn from_instructions(instructions: &[Instruction]) -> Self {
        Self { instructions: instructions.to_vec() }
    }
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RegIdx(u8);

impl From<u8> for RegIdx {
    fn from(n: u8) -> Self {
        Self(n)
    }
}

#[derive(Clone, Copy)]
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
    Noop {},
    Halt {},
}

impl Instruction {
    pub fn try_from_line(line: &str) -> Result<Option<Self>, ()> {
        lazy_static! {
            static ref LINE_RE: Regex = Regex::new(r"(?x)
            (?P<opcode>[a-zA-z]{3,6})
            (?P<args>\s+
                (
                    (?P<immediate>-?\d+)
                    |
                    (
                        (r(?P<r1>\d{1,2}))
                        (\s+r(?P<r2>\d{1,2}))?
                        (\s+r(?P<r3>\d{1,2}))?
                    )
                )
            )").expect("TODO");
        }
        let captures = LINE_RE.captures(line).ok_or(())?;
        let opcode_match = captures.name("opcode");
        if opcode_match.is_none() {
            return Ok(None);
        }
        match &opcode_match.unwrap().as_str().to_lowercase()[..] {
            "arrins" => {
                let val = captures.name("r1");
                let arr = captures.name("r2");
                let idx = captures.name("r3");
                if let (Some(val), Some(arr), Some(idx)) = (val, arr, idx) {
                    Ok(Some(Self::ArrIns { 
                        val: val.as_str().parse::<u8>().expect("TODO").into(),
                        arr: arr.as_str().parse::<u8>().expect("TODO").into(),
                        idx: idx.as_str().parse::<u8>().expect("TODO").into(),
                    }))
                } else {
                    Err(())
                }
            }
            "arrget" => {
                let dst = captures.name("r1");
                let arr = captures.name("r2");
                let idx = captures.name("r3");
                if let (Some(dst), Some(arr), Some(idx)) = (dst, arr, idx) {
                    Ok(Some(Self::ArrGet { 
                        dst: dst.as_str().parse::<u8>().expect("TODO").into(),
                        arr: arr.as_str().parse::<u8>().expect("TODO").into(),
                        idx: idx.as_str().parse::<u8>().expect("TODO").into(),
                    }))
                } else {
                    Err(())
                }
            }
            "movimm" => {
                let dst = captures.name("r1");
                let imm = captures.name("immediate");
                if let (Some(dst), Some(imm)) = (dst, imm) {
                    Ok(Some(Self::MovImm {
                        dst: dst.as_str().parse::<u8>().expect("TODO").into(),
                        imm: imm.as_str().parse().expect("TODO"),
                    }))
                } else {
                    Err(())
                }
            }
            "mov" => {
                let dst = captures.name("r1");
                let src = captures.name("r2");
                if let (Some(dst), Some(src)) = (dst, src) {
                    Ok(Some(Self::Mov {
                        dst: dst.as_str().parse::<u8>().expect("TODO").into(),
                        src: src.as_str().parse::<u8>().expect("TODO").into(),
                    }))
                } else {
                    Err(())
                }
            }
            "jmp" => {
                let adr = captures.name("r1");
                if let Some(adr) = adr {
                    Ok(Some(Self::Jmp {
                        adr: adr.as_str().parse::<u8>().expect("TODO").into(),
                    }))
                } else {
                    Err(())
                }
            }
            "jeq" => {
                let adr = captures.name("r1");
                let v0 = captures.name("r2");
                let v1 = captures.name("r3");
                if let (Some(adr), Some(v0), Some(v1)) = (adr, v0, v1) {
                    Ok(Some(Self::Jeq {
                        adr: adr.as_str().parse::<u8>().expect("TODO").into(),
                        v0: v0.as_str().parse::<u8>().expect("TODO").into(),
                        v1: v1.as_str().parse::<u8>().expect("TODO").into(),
                    }))
                } else {
                    Err(())
                }
            }
            "jne" => {
                let adr = captures.name("r1");
                let v0 = captures.name("r2");
                let v1 = captures.name("r3");
                if let (Some(adr), Some(v0), Some(v1)) = (adr, v0, v1) {
                    Ok(Some(Self::Jne {
                        adr: adr.as_str().parse::<u8>().expect("TODO").into(),
                        v0: v0.as_str().parse::<u8>().expect("TODO").into(),
                        v1: v1.as_str().parse::<u8>().expect("TODO").into(),
                    }))
                } else {
                    Err(())
                }
            }
            "jgt" => {
                let adr = captures.name("r1");
                let v0 = captures.name("r2");
                let v1 = captures.name("r3");
                if let (Some(adr), Some(v0), Some(v1)) = (adr, v0, v1) {
                    Ok(Some(Self::Jgt {
                        adr: adr.as_str().parse::<u8>().expect("TODO").into(),
                        v0: v0.as_str().parse::<u8>().expect("TODO").into(),
                        v1: v1.as_str().parse::<u8>().expect("TODO").into(),
                    }))
                } else {
                    Err(())
                }
            }
            "jge" => {
                let adr = captures.name("r1");
                let v0 = captures.name("r2");
                let v1 = captures.name("r3");
                if let (Some(adr), Some(v0), Some(v1)) = (adr, v0, v1) {
                    Ok(Some(Self::Jge {
                        adr: adr.as_str().parse::<u8>().expect("TODO").into(),
                        v0: v0.as_str().parse::<u8>().expect("TODO").into(),
                        v1: v1.as_str().parse::<u8>().expect("TODO").into(),
                    }))
                } else {
                    Err(())
                }
            }
            "jlt" => {
                let adr = captures.name("r1");
                let v0 = captures.name("r2");
                let v1 = captures.name("r3");
                if let (Some(adr), Some(v0), Some(v1)) = (adr, v0, v1) {
                    Ok(Some(Self::Jlt {
                        adr: adr.as_str().parse::<u8>().expect("TODO").into(),
                        v0: v0.as_str().parse::<u8>().expect("TODO").into(),
                        v1: v1.as_str().parse::<u8>().expect("TODO").into(),
                    }))
                } else {
                    Err(())
                }
            }
            "jle" => {
                let adr = captures.name("r1");
                let v0 = captures.name("r2");
                let v1 = captures.name("r3");
                if let (Some(adr), Some(v0), Some(v1)) = (adr, v0, v1) {
                    Ok(Some(Self::Jle {
                        adr: adr.as_str().parse::<u8>().expect("TODO").into(),
                        v0: v0.as_str().parse::<u8>().expect("TODO").into(),
                        v1: v1.as_str().parse::<u8>().expect("TODO").into(),
                    }))
                } else {
                    Err(())
                }
            }
            "add" => {
                let dst = captures.name("r1");
                let v0 = captures.name("r2");
                let v1 = captures.name("r3");
                if let (Some(dst), Some(v0), Some(v1)) = (dst, v0, v1) {
                    Ok(Some(Self::Add {
                        dst: dst.as_str().parse::<u8>().expect("TODO").into(),
                        v0: v0.as_str().parse::<u8>().expect("TODO").into(),
                        v1: v1.as_str().parse::<u8>().expect("TODO").into(),
                    }))
                } else {
                    Err(())
                }
            }
            "mul" => {
                let dst = captures.name("r1");
                let v0 = captures.name("r2");
                let v1 = captures.name("r3");
                if let (Some(dst), Some(v0), Some(v1)) = (dst, v0, v1) {
                    Ok(Some(Self::Mul {
                        dst: dst.as_str().parse::<u8>().expect("TODO").into(),
                        v0: v0.as_str().parse::<u8>().expect("TODO").into(),
                        v1: v1.as_str().parse::<u8>().expect("TODO").into(),
                    }))
                } else {
                    Err(())
                }
            }
            "div" => {
                let dst = captures.name("r1");
                let v0 = captures.name("r2");
                let v1 = captures.name("r3");
                if let (Some(dst), Some(v0), Some(v1)) = (dst, v0, v1) {
                    Ok(Some(Self::Div {
                        dst: dst.as_str().parse::<u8>().expect("TODO").into(),
                        v0: v0.as_str().parse::<u8>().expect("TODO").into(),
                        v1: v1.as_str().parse::<u8>().expect("TODO").into(),
                    }))
                } else {
                    Err(())
                }
            }
            "mod" => {
                let dst = captures.name("r1");
                let v0 = captures.name("r2");
                let v1 = captures.name("r3");
                if let (Some(dst), Some(v0), Some(v1)) = (dst, v0, v1) {
                    Ok(Some(Self::Mod {
                        dst: dst.as_str().parse::<u8>().expect("TODO").into(),
                        v0: v0.as_str().parse::<u8>().expect("TODO").into(),
                        v1: v1.as_str().parse::<u8>().expect("TODO").into(),
                    }))
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse {
        use super::*;

        fn check_arrins(line: &str, idx0: u8, idx1: u8, idx2: u8) {
            if let Instruction::ArrIns { val, arr, idx } = Instruction::try_from_line(line).unwrap().unwrap() {
                assert_eq!(val, RegIdx::from(idx0));
                assert_eq!(arr, RegIdx::from(idx1));
                assert_eq!(idx, RegIdx::from(idx2));
            } else {
                panic!("Wrong opcode parse")
            }
        }

        #[test]
        fn parse_arrins() {
            check_arrins("ArrIns r0 r1 r2", 0, 1, 2);
            check_arrins("arrins r0 r0 r00", 0, 0, 0);
            check_arrins("ARRINS r0 r0 r0", 0, 0, 0);
        }

        #[test]
        fn parse_arrget() {
            let line = "arrget r0 r1 r2";
            if let Instruction::ArrGet { dst, arr, idx } = Instruction::try_from_line(line).unwrap().unwrap() {
                assert_eq!(dst, RegIdx::from(0));
                assert_eq!(arr, RegIdx::from(1));
                assert_eq!(idx, RegIdx::from(2));
            } else {
                panic!("Could not parse line")
            }
        }

        fn check_movimm(line: &str, reg: u8, expected_imm: i32) {
            if let Instruction::MovImm { dst, imm } = Instruction::try_from_line(line).unwrap().unwrap() {
                assert_eq!(dst, RegIdx::from(reg));
                assert_eq!(imm, expected_imm);
            } else {
                panic!("Could not parse line");
            }
        }

        #[test]
        fn parse_movimm() {
            check_movimm("movimm r0 12345", 0, 12345);
            check_movimm("movimm r0 -12345", 0, -12345);
            check_movimm("movimm r10 00000", 10, 0);
        }

        fn check_mov(line: &str, idx0: u8, idx1: u8) {
            if let Instruction::Mov { dst, src } = Instruction::try_from_line(line).unwrap().unwrap() {
                assert_eq!(dst, RegIdx::from(idx0));
                assert_eq!(src, RegIdx::from(idx1));
            } else {
                panic!("Could not parse line");
            }
        }

        #[test]
        fn parse_mov() {
            check_mov("mov r11 r12", 11, 12);
            check_mov("mov r02 r1", 2, 1);
        }

        #[test]
        fn parse_jmp() {
            let line = "jmp r0";
            if let Instruction::Jmp { adr } = Instruction::try_from_line(line).unwrap().unwrap() {
                assert_eq!(adr, RegIdx::from(0));
            } else {
                panic!("Could not parse line");
            }
        }

        #[test]
        fn parse_jeq() {
            let line = "jeq r0 r1 r2";
            if let Instruction::Jeq { adr, v0, v1 } = Instruction::try_from_line(line).unwrap().unwrap() {
                assert_eq!(adr, RegIdx::from(0));
                assert_eq!(v0, RegIdx::from(1));
                assert_eq!(v1, RegIdx::from(2));
            } else {
                panic!("Parsed as incorrect opcode");
            }
        }

        #[test]
        fn parse_jne() {
            let line = "jne r0 r1 r2";
            if let Instruction::Jne { adr, v0, v1 } = Instruction::try_from_line(line).unwrap().unwrap() {
                assert_eq!(adr, RegIdx::from(0));
                assert_eq!(v0, RegIdx::from(1));
                assert_eq!(v1, RegIdx::from(2));
            } else {
                panic!("Parsed as incorrect opcode");
            }
        }

        #[test]
        fn parse_jgt() {
            let line = "jgt r0 r1 r2";
            if let Instruction::Jgt { adr, v0, v1 } = Instruction::try_from_line(line).unwrap().unwrap() {
                assert_eq!(adr, RegIdx::from(0));
                assert_eq!(v0, RegIdx::from(1));
                assert_eq!(v1, RegIdx::from(2));
            } else {
                panic!("Parsed as incorrect opcode");
            }
        }

        #[test]
        fn parse_jge() {
            let line = "jge r0 r1 r2";
            if let Instruction::Jge { adr, v0, v1 } = Instruction::try_from_line(line).unwrap().unwrap() {
                assert_eq!(adr, RegIdx::from(0));
                assert_eq!(v0, RegIdx::from(1));
                assert_eq!(v1, RegIdx::from(2));
            } else {
                panic!("Parsed as incorrect opcode");
            }
        }

        #[test]
        fn parse_jlt() {
            let line = "jlt r0 r1 r2";
            if let Instruction::Jlt { adr, v0, v1 } = Instruction::try_from_line(line).unwrap().unwrap() {
                assert_eq!(adr, RegIdx::from(0));
                assert_eq!(v0, RegIdx::from(1));
                assert_eq!(v1, RegIdx::from(2));
            } else {
                panic!("Parsed as incorrect opcode");
            }
        }

        #[test]
        fn parse_jle() {
            let line = "jle r0 r1 r2";
            if let Instruction::Jle { adr, v0, v1 } = Instruction::try_from_line(line).unwrap().unwrap() {
                assert_eq!(adr, RegIdx::from(0));
                assert_eq!(v0, RegIdx::from(1));
                assert_eq!(v1, RegIdx::from(2));
            } else {
                panic!("Parsed as incorrect opcode");
            }
        }

        #[test]
        fn parse_program() {
            let prog = "mov r0 r1\nmovimm r2 123\n\nadd r0 r10";
            let result = Program::try_from_str(prog);
            assert!(result.is_ok());
        }

        #[test]
        fn parse_empty_line() {
            let result = Instruction::try_from_line("");
            assert!(result.is_ok());
            assert!(result.unwrap().is_none());
        }

        #[test]
        fn parse_bad_opcode() {
            let result = Instruction::try_from_line("bad r0 r1 r2");
            assert!(result.is_err());
        }

        #[test]
        fn parse_halt() {
            panic!("Unimplemented");
        }

        #[test]
        fn parse_noop() {
            panic!("Unimplemented");
        }
    }

    mod execute {
        use super::*;

        #[test]
        fn exec_arrins_allocate() {
            let arr_register = RegIdx::from(0);
            let idx_register = RegIdx::from(1);
            let val_register = RegIdx::from(2);

            let program = Program::from_instructions(&[
                Instruction::ArrIns { arr: arr_register, idx: idx_register, val: val_register }
            ]);

            let mut state = ExecutionState::default();
            state.register_write(arr_register, 0);
            state.register_write(idx_register, 0);
            state.register_write(val_register, 123);
            let mut program_instance = ProgramInstance::new(program, state);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.array_read(0, 0), 123);
        }

        #[test]
        fn exec_arrins_overwrite() {
            let arr_register = RegIdx::from(0);
            let idx_register = RegIdx::from(1);
            let val_register = RegIdx::from(2);

            let program = Program::from_instructions(&[
                Instruction::ArrIns { arr: arr_register, idx: idx_register, val: val_register },
            ]);

            let mut state = ExecutionState::default();
            state.register_write(arr_register, 0);
            state.register_write(idx_register, 0);
            state.register_write(val_register, 321);
            state.array_insert(0, 0, 123);

            let mut program_instance = ProgramInstance::new(program, state);
            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.array_read(0, 0), 321);
        }

        #[test]
        fn exec_arrget_empty() {
            let dst_register = RegIdx::from(0);
            let arr_register = RegIdx::from(1);
            let idx_register = RegIdx::from(2);

            let program = Program::from_instructions(&[
                Instruction::ArrGet { dst: dst_register, arr: arr_register, idx: idx_register },
            ]);

            let mut state = ExecutionState::default();
            state.register_write(dst_register, 345);
            state.register_write(arr_register, 123);
            state.register_write(idx_register, 987);

            let mut program_instance = ProgramInstance::new(program, state);
            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.register_read(dst_register), 0);
        }

        #[test]
        fn exec_arrget_present() {
            let dst_register = RegIdx::from(0);
            let arr_register = RegIdx::from(1);
            let idx_register = RegIdx::from(2);

            let program = Program::from_instructions(&[
                Instruction::ArrGet { dst: dst_register, arr: arr_register, idx: idx_register },
            ]);

            let mut state = ExecutionState::default();
            state.register_write(dst_register, 345);
            state.register_write(arr_register, 123);
            state.register_write(idx_register, 987);
            state.array_insert(123, 987, 543);

            let mut program_instance = ProgramInstance::new(program, state);
            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.register_read(dst_register), 543);
        }

        #[test]
        fn exec_movimm() {
            let dst0_register = RegIdx::from(0);
            let dst1_register = RegIdx::from(1);
            let dst2_register = RegIdx::from(2);

            let val0 = 1i64;
            let val1 = 34i64;
            let val2 = -123i64;
            let val3 = 0i64;

            let program = Program::from_instructions(&[
                Instruction::MovImm { dst: dst0_register, imm: val0 as i32},
                Instruction::MovImm { dst: dst1_register, imm: val1 as i32 },
                Instruction::MovImm { dst: dst2_register, imm: val2 as i32 },
                Instruction::MovImm { dst: dst0_register, imm: val3 as i32 },
            ]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.register_read(dst0_register), val0);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.register_read(dst0_register), val0);
            assert_eq!(program_instance.state.register_read(dst1_register), val1);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.register_read(dst0_register), val0);
            assert_eq!(program_instance.state.register_read(dst1_register), val1);
            assert_eq!(program_instance.state.register_read(dst2_register), val2);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_ne!(program_instance.state.register_read(dst0_register), val0);
            assert_eq!(program_instance.state.register_read(dst1_register), val1);
            assert_eq!(program_instance.state.register_read(dst2_register), val2);
            assert_eq!(program_instance.state.register_read(dst0_register), val3);
        }

        #[test]
        fn exec_mov() {
            let register0 = RegIdx::from(0);
            let register1 = RegIdx::from(1);
            let register2 = RegIdx::from(2);

            let val0 = 123i64;

            let program = Program::from_instructions(&[
                Instruction::MovImm { dst: register0, imm: val0 as i32 },
                Instruction::Mov { dst: register1, src: register0 },
                Instruction::Mov { dst: register0, src: register2 },
                Instruction::Mov { dst: register2, src: register1 },
            ]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.register_read(register0), val0);
            assert_eq!(program_instance.state.register_read(register1), 0);
            assert_eq!(program_instance.state.register_read(register2), 0);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.register_read(register0), val0);
            assert_eq!(program_instance.state.register_read(register1), val0);
            assert_eq!(program_instance.state.register_read(register2), 0);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.register_read(register0), 0);
            assert_eq!(program_instance.state.register_read(register1), val0);
            assert_eq!(program_instance.state.register_read(register2), 0);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.register_read(register0), 0);
            assert_eq!(program_instance.state.register_read(register1), val0);
            assert_eq!(program_instance.state.register_read(register2), val0);
        }

        #[test]
        fn exec_jmp() {
            let target_register = RegIdx::from(10);
            let target_addr = 2i64;

            let program = Program::from_instructions(&[
                Instruction::MovImm { dst: target_register, imm: target_addr as i32 },
                Instruction::Noop { },
                Instruction::Noop { },
                Instruction::Noop { },
                Instruction::Jmp { adr: target_register },
            ]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            assert_eq!(program_instance.state.register_read(RP_IDX), 0);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));

            assert_eq!(program_instance.state.register_read(RP_IDX), 1);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));

            assert_eq!(program_instance.state.register_read(RP_IDX), 2);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));

            assert_eq!(program_instance.state.register_read(RP_IDX), 3);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));

            assert_eq!(program_instance.state.register_read(RP_IDX), 4);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));

            assert_eq!(program_instance.state.register_read(RP_IDX), target_addr);
        }

        #[test]
        fn exec_jmp_oob() {
            let target_register = RegIdx::from(10);
            let target_addr = 123i64;

            let program = Program::from_instructions(&[
                Instruction::MovImm { dst: target_register, imm: target_addr as i32 },
                Instruction::Jmp { adr: target_register },
            ]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            assert_eq!(program_instance.state.register_read(RP_IDX), 0);
            assert_eq!(program_instance.execute_step(), Ok(true));
            assert_eq!(program_instance.state.register_read(RP_IDX), 1);
            assert_eq!(program_instance.execute_step(), Ok(true));
            assert_eq!(program_instance.state.register_read(RP_IDX), target_addr);
            assert_eq!(program_instance.execute_step(), Err(()));
        }

        #[test]
        fn exec_jeq() {
            let eq_target_register = RegIdx::from(10);
            let r0 = RegIdx::from(0);
            let r1 = RegIdx::from(1);
            let r2 = RegIdx::from(2);

            let eq_target = 987i64;
            let v1 = 123i64;
            let v2 = 321i64;

            let program = Program::from_instructions(&[
                Instruction::MovImm { dst: eq_target_register, imm: eq_target as i32 },
                Instruction::MovImm { dst: r0, imm: v1 as i32 },
                Instruction::MovImm { dst: r1, imm: v2 as i32 },
                Instruction::MovImm { dst: r2, imm: v1 as i32 },
                Instruction::Jeq { adr: eq_target_register, v0: r0, v1: r1 },
                Instruction::Jeq { adr: eq_target_register, v0: r0, v1: r2 },
            ]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            for _ in 0..4 {
                assert_eq!(program_instance.execute_step(), Ok(true));
            }
            assert_eq!(program_instance.state.register_read(RP_IDX), 4);
            assert_eq!(program_instance.execute_step(), Ok(true));
            assert_eq!(program_instance.state.register_read(RP_IDX), 5);
            assert_eq!(program_instance.execute_step(), Ok(true));
            assert_eq!(program_instance.state.register_read(RP_IDX), eq_target);
        }

        #[test]
        fn exec_jne() {
            panic!("Unimplemented")
        }

        #[test]
        fn exec_jgt() {
            panic!("Unimplemented")
        }

        #[test]
        fn exec_jge() {
            panic!("Unimplemented")
        }

        #[test]
        fn exec_jlt() {
            panic!("Unimplemented")
        }

        #[test]
        fn exec_jle() {
            panic!("Unimplemented")
        }

        #[test]
        fn exec_add_overflow() {
            panic!("Unimplemented")
        }

        #[test]
        fn exec_add_negative() {
            panic!("Unimplemented")
        }

        #[test]
        fn exec_mul() {
            panic!("Unimplemented")
        }

        #[test]
        fn exec_div() {
            panic!("Unimplemented")
        }

        #[test]
        fn exec_mod() {
            panic!("Unimplemented")
        }

        #[test]
        fn exec_halt() {
            panic!("Unimplemented");
        }

        #[test]
        fn exec_noop() {
            panic!("Unimplemented");
        }

        #[test]
        fn exec_empty_program() {
            panic!("Unimplemented");
        }

        #[test]
        fn exec_end_of_code() {
            panic!("Unimplemented");
        }

        #[test]
        fn exec_reused_register() {
            panic!("Unimplemented");
        }
    }
}
