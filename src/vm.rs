use lazy_static::lazy_static;
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    ParseError,
    ExecutionError,
}

const RP_IDX: RegIdx = RegIdx(15);

pub struct ProgramInstance {
    program: Program,
    state: ExecutionState,
}

impl ProgramInstance {
    pub fn new(program: Program, state: ExecutionState) -> Self {
        Self { program, state }
    }

    pub fn execute_step(&mut self) -> Result<bool, Error> {
        let instruction = self
            .program
            .instructions
            .get(self.state.register_read(RP_IDX) as usize)
            .ok_or(Error::ExecutionError)?;
        match instruction {
            Instruction::ArrIns { arr, idx, val } => {
                self.state.array_insert(
                    self.state.register_read(*arr) as u64,
                    self.state.register_read(*idx) as u64,
                    self.state.register_read(*val),
                );
                self.state.incremement_rp();
                Ok(true)
            }
            Instruction::ArrGet { dst, arr, idx } => {
                self.state.register_write(
                    *dst,
                    self.state.array_read(
                        self.state.register_read(*arr) as u64,
                        self.state.register_read(*idx) as u64,
                    ),
                );
                self.state.incremement_rp();
                Ok(true)
            }
            Instruction::MovImm { dst, imm } => {
                self.state.register_write(*dst, *imm as i64);
                self.state.incremement_rp();
                Ok(true)
            }
            Instruction::Mov { dst, src } => {
                self.state
                    .register_write(*dst, self.state.register_read(*src));
                self.state.incremement_rp();
                Ok(true)
            }
            Instruction::Jmp { adr } => {
                self.state
                    .register_write(RP_IDX, self.state.register_read(*adr));
                Ok(true)
            }
            Instruction::Jeq { adr, v0, v1 } => {
                if self.state.register_read(*v0) == self.state.register_read(*v1) {
                    self.state
                        .register_write(RP_IDX, self.state.register_read(*adr));
                } else {
                    self.state.incremement_rp();
                }
                Ok(true)
            }
            Instruction::Jne { adr, v0, v1 } => {
                if self.state.register_read(*v0) != self.state.register_read(*v1) {
                    self.state
                        .register_write(RP_IDX, self.state.register_read(*adr));
                } else {
                    self.state.incremement_rp();
                }
                Ok(true)
            }
            Instruction::Jgt { adr, v0, v1 } => {
                if self.state.register_read(*v0) > self.state.register_read(*v1) {
                    self.state
                        .register_write(RP_IDX, self.state.register_read(*adr));
                } else {
                    self.state.incremement_rp();
                }
                Ok(true)
            }
            Instruction::Jge { adr, v0, v1 } => {
                if self.state.register_read(*v0) >= self.state.register_read(*v1) {
                    self.state
                        .register_write(RP_IDX, self.state.register_read(*adr));
                } else {
                    self.state.incremement_rp();
                }
                Ok(true)
            }
            Instruction::Jlt { adr, v0, v1 } => {
                if self.state.register_read(*v0) < self.state.register_read(*v1) {
                    self.state
                        .register_write(RP_IDX, self.state.register_read(*adr));
                } else {
                    self.state.incremement_rp();
                }
                Ok(true)
            }
            Instruction::Jle { adr, v0, v1 } => {
                if self.state.register_read(*v0) <= self.state.register_read(*v1) {
                    self.state
                        .register_write(RP_IDX, self.state.register_read(*adr));
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
            Instruction::Halt {} => {
                self.state.incremement_rp();
                Ok(false)
            }
            Instruction::Noop {} => {
                self.state.incremement_rp();
                Ok(true)
            }
        }
    }

    pub(crate) fn state(&self) -> &ExecutionState {
        &self.state
    }

    #[cfg(test)]
    pub(crate) fn state_mut(&mut self) -> &mut ExecutionState {
        &mut self.state
    }
}

#[derive(Clone)]
pub struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    pub fn try_from_str(s: &str) -> Result<Self, Error> {
        let instructions: Vec<_> = s
            .lines()
            .filter_map(|line| Instruction::try_from_line(line).expect("TODO"))
            .collect();
        Ok(Self { instructions })
    }

    pub fn from_instructions(instructions: &[Instruction]) -> Self {
        Self {
            instructions: instructions.to_vec(),
        }
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
        self.arrays.borrow_mut().entry(arr).or_default().get(idx)
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

    pub fn iter_touched_values(&self, arr: u64) -> Box<dyn Iterator<Item = (u64, i64)>> {
        Box::new(
            self.arrays
                .borrow()
                .get(&arr)
                .map(|array| &array.0)
                .unwrap_or(&HashMap::default())
                .clone()
                .into_iter(),
        )
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
    pub fn try_from_line(line: &str) -> Result<Option<Self>, Error> {
        lazy_static! {
            static ref LINE_RE: Regex = Regex::new(
                r"(?x)
            (?P<opcode>[a-zA-z]{3,6})
            (?P<args>\s+
                (
                    (r(?P<r1>\d{1,2}))
                    (
                        (\s+(?P<immediate>\-?\d+))
                        |
                        (
                            (\s+r(?P<r2>\d{1,2}))?
                            (\s+r(?P<r3>\d{1,2}))?
                        )
                    )
                )
            )?"
            )
            .expect("TODO");
        }

        if line.chars().all(|c| c.is_whitespace()) {
            return Ok(None);
        }

        let captures = LINE_RE.captures(line).ok_or(Error::ParseError)?;
        let opcode_match = captures.name("opcode");
        if opcode_match.is_none() {
            return Ok(None);
        }
        match &opcode_match.expect("TODO").as_str().to_lowercase()[..] {
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
                    Err(Error::ParseError)
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
                    Err(Error::ParseError)
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
                    Err(Error::ParseError)
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
                    Err(Error::ParseError)
                }
            }
            "jmp" => {
                let adr = captures.name("r1");
                if let Some(adr) = adr {
                    Ok(Some(Self::Jmp {
                        adr: adr.as_str().parse::<u8>().expect("TODO").into(),
                    }))
                } else {
                    Err(Error::ParseError)
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
                    Err(Error::ParseError)
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
                    Err(Error::ParseError)
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
                    Err(Error::ParseError)
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
                    Err(Error::ParseError)
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
                    Err(Error::ParseError)
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
                    Err(Error::ParseError)
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
                    Err(Error::ParseError)
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
                    Err(Error::ParseError)
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
                    Err(Error::ParseError)
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
                    Err(Error::ParseError)
                }
            }
            "halt" => Ok(Some(Self::Halt {})),
            "noop" => Ok(Some(Self::Noop {})),
            _ => Err(Error::ParseError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const R0_IDX: RegIdx = RegIdx(0);
    const R1_IDX: RegIdx = RegIdx(1);
    const R2_IDX: RegIdx = RegIdx(2);
    const R10_IDX: RegIdx = RegIdx(10);

    mod parse {
        use super::*;

        fn check_arrins(line: &str, idx0: u8, idx1: u8, idx2: u8) {
            if let Instruction::ArrIns { val, arr, idx } = Instruction::try_from_line(line)
                .expect("TODO")
                .expect("TODO")
            {
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
            if let Instruction::ArrGet { dst, arr, idx } = Instruction::try_from_line(line)
                .expect("TODO")
                .expect("TODO")
            {
                assert_eq!(dst, R0_IDX);
                assert_eq!(arr, R1_IDX);
                assert_eq!(idx, R2_IDX);
            } else {
                panic!("Could not parse line")
            }
        }

        fn check_movimm(line: &str, reg: u8, expected_imm: i32) {
            if let Instruction::MovImm { dst, imm } = Instruction::try_from_line(line)
                .expect("TODO")
                .expect("TODO")
            {
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
            if let Instruction::Mov { dst, src } = Instruction::try_from_line(line)
                .expect("TODO")
                .expect("TODO")
            {
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
            if let Instruction::Jmp { adr } = Instruction::try_from_line(line)
                .expect("TODO")
                .expect("TODO")
            {
                assert_eq!(adr, R0_IDX);
            } else {
                panic!("Could not parse line");
            }
        }

        #[test]
        fn parse_jeq() {
            let line = "jeq r0 r1 r2";
            if let Instruction::Jeq { adr, v0, v1 } = Instruction::try_from_line(line)
                .expect("TODO")
                .expect("TODO")
            {
                assert_eq!(adr, R0_IDX);
                assert_eq!(v0, R1_IDX);
                assert_eq!(v1, R2_IDX);
            } else {
                panic!("Parsed as incorrect opcode");
            }
        }

        #[test]
        fn parse_jne() {
            let line = "jne r0 r1 r2";
            if let Instruction::Jne { adr, v0, v1 } = Instruction::try_from_line(line)
                .expect("TODO")
                .expect("TODO")
            {
                assert_eq!(adr, R0_IDX);
                assert_eq!(v0, R1_IDX);
                assert_eq!(v1, R2_IDX);
            } else {
                panic!("Parsed as incorrect opcode");
            }
        }

        #[test]
        fn parse_jgt() {
            let line = "jgt r0 r1 r2";
            if let Instruction::Jgt { adr, v0, v1 } = Instruction::try_from_line(line)
                .expect("TODO")
                .expect("TODO")
            {
                assert_eq!(adr, R0_IDX);
                assert_eq!(v0, R1_IDX);
                assert_eq!(v1, R2_IDX);
            } else {
                panic!("Parsed as incorrect opcode");
            }
        }

        #[test]
        fn parse_jge() {
            let line = "jge r0 r1 r2";
            if let Instruction::Jge { adr, v0, v1 } = Instruction::try_from_line(line)
                .expect("TODO")
                .expect("TODO")
            {
                assert_eq!(adr, R0_IDX);
                assert_eq!(v0, R1_IDX);
                assert_eq!(v1, R2_IDX);
            } else {
                panic!("Parsed as incorrect opcode");
            }
        }

        #[test]
        fn parse_jlt() {
            let line = "jlt r0 r1 r2";
            if let Instruction::Jlt { adr, v0, v1 } = Instruction::try_from_line(line)
                .expect("TODO")
                .expect("TODO")
            {
                assert_eq!(adr, R0_IDX);
                assert_eq!(v0, R1_IDX);
                assert_eq!(v1, R2_IDX);
            } else {
                panic!("Parsed as incorrect opcode");
            }
        }

        #[test]
        fn parse_jle() {
            let line = "jle r0 r1 r2";
            if let Instruction::Jle { adr, v0, v1 } = Instruction::try_from_line(line)
                .expect("TODO")
                .expect("TODO")
            {
                assert_eq!(adr, R0_IDX);
                assert_eq!(v0, R1_IDX);
                assert_eq!(v1, R2_IDX);
            } else {
                panic!("Parsed as incorrect opcode");
            }
        }

        #[test]
        fn parse_program() {
            let prog = "mov r0 r1\nmovimm r2 123\n\nadd r0 r2 r10";
            let result = Program::try_from_str(prog);
            assert!(result.is_ok());
        }

        #[test]
        fn parse_empty_line() {
            let result = Instruction::try_from_line("");
            assert!(result.is_ok());
            assert!(result.expect("TODO").is_none());
        }

        #[test]
        fn parse_bad_opcode() {
            let result = Instruction::try_from_line("bad r0 r1 r2");
            assert!(result.is_err());
        }

        #[test]
        fn parse_halt() {
            let result = Instruction::try_from_line("halt");
            if let Ok(Some(Instruction::Halt {})) = result {
            } else {
                panic!("Failed to parse Halt")
            }
        }

        #[test]
        fn parse_noop() {
            let result = Instruction::try_from_line("NOOP");
            if let Ok(Some(Instruction::Noop {})) = result {
            } else {
                panic!("Failed to parse Noop")
            }
        }
    }

    mod execute {
        use super::*;

        #[test]
        fn exec_arrins_allocate() {
            let arr_register = R0_IDX;
            let idx_register = R1_IDX;
            let val_register = R2_IDX;

            let program = Program::from_instructions(&[Instruction::ArrIns {
                arr: arr_register,
                idx: idx_register,
                val: val_register,
            }]);

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
            let arr_register = R0_IDX;
            let idx_register = R1_IDX;
            let val_register = R2_IDX;

            let program = Program::from_instructions(&[Instruction::ArrIns {
                arr: arr_register,
                idx: idx_register,
                val: val_register,
            }]);

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
            let dst_register = R0_IDX;
            let arr_register = R1_IDX;
            let idx_register = R2_IDX;

            let program = Program::from_instructions(&[Instruction::ArrGet {
                dst: dst_register,
                arr: arr_register,
                idx: idx_register,
            }]);

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
            let dst_register = R0_IDX;
            let arr_register = R1_IDX;
            let idx_register = R2_IDX;

            let program = Program::from_instructions(&[Instruction::ArrGet {
                dst: dst_register,
                arr: arr_register,
                idx: idx_register,
            }]);

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
            let val0 = 1i64;
            let val1 = 34i64;
            let val2 = -123i64;
            let val3 = 0i64;

            let program = Program::from_instructions(&[
                Instruction::MovImm {
                    dst: R0_IDX,
                    imm: val0 as i32,
                },
                Instruction::MovImm {
                    dst: R1_IDX,
                    imm: val1 as i32,
                },
                Instruction::MovImm {
                    dst: R2_IDX,
                    imm: val2 as i32,
                },
                Instruction::MovImm {
                    dst: R0_IDX,
                    imm: val3 as i32,
                },
            ]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.register_read(R0_IDX), val0);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.register_read(R0_IDX), val0);
            assert_eq!(program_instance.state.register_read(R1_IDX), val1);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.register_read(R0_IDX), val0);
            assert_eq!(program_instance.state.register_read(R1_IDX), val1);
            assert_eq!(program_instance.state.register_read(R2_IDX), val2);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_ne!(program_instance.state.register_read(R0_IDX), val0);
            assert_eq!(program_instance.state.register_read(R1_IDX), val1);
            assert_eq!(program_instance.state.register_read(R2_IDX), val2);
            assert_eq!(program_instance.state.register_read(R0_IDX), val3);
        }

        #[test]
        fn exec_mov() {
            let val0 = 123i64;

            let program = Program::from_instructions(&[
                Instruction::MovImm {
                    dst: R0_IDX,
                    imm: val0 as i32,
                },
                Instruction::Mov {
                    dst: R1_IDX,
                    src: R0_IDX,
                },
                Instruction::Mov {
                    dst: R0_IDX,
                    src: R2_IDX,
                },
                Instruction::Mov {
                    dst: R2_IDX,
                    src: R1_IDX,
                },
            ]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.register_read(R0_IDX), val0);
            assert_eq!(program_instance.state.register_read(R1_IDX), 0);
            assert_eq!(program_instance.state.register_read(R2_IDX), 0);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.register_read(R0_IDX), val0);
            assert_eq!(program_instance.state.register_read(R1_IDX), val0);
            assert_eq!(program_instance.state.register_read(R2_IDX), 0);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.register_read(R0_IDX), 0);
            assert_eq!(program_instance.state.register_read(R1_IDX), val0);
            assert_eq!(program_instance.state.register_read(R2_IDX), 0);

            let step_result = program_instance.execute_step();
            assert_eq!(step_result, Ok(true));
            assert_eq!(program_instance.state.register_read(R0_IDX), 0);
            assert_eq!(program_instance.state.register_read(R1_IDX), val0);
            assert_eq!(program_instance.state.register_read(R2_IDX), val0);
        }

        #[test]
        fn exec_jmp() {
            let target_register = R10_IDX;
            let target_addr = 2i64;

            let program = Program::from_instructions(&[
                Instruction::MovImm {
                    dst: target_register,
                    imm: target_addr as i32,
                },
                Instruction::Noop {},
                Instruction::Noop {},
                Instruction::Noop {},
                Instruction::Jmp {
                    adr: target_register,
                },
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
            let target_register = R10_IDX;
            let target_addr = 123i64;

            let program = Program::from_instructions(&[
                Instruction::MovImm {
                    dst: target_register,
                    imm: target_addr as i32,
                },
                Instruction::Jmp {
                    adr: target_register,
                },
            ]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            assert_eq!(program_instance.state.register_read(RP_IDX), 0);
            assert_eq!(program_instance.execute_step(), Ok(true));
            assert_eq!(program_instance.state.register_read(RP_IDX), 1);
            assert_eq!(program_instance.execute_step(), Ok(true));
            assert_eq!(program_instance.state.register_read(RP_IDX), target_addr);
            assert_eq!(program_instance.execute_step(), Err(Error::ExecutionError));
        }

        #[test]
        fn exec_jeq() {
            let eq_target_register = R10_IDX;
            let eq_target = 987i64;
            let v1 = 123i64;
            let v2 = 321i64;

            let program = Program::from_instructions(&[
                Instruction::MovImm {
                    dst: eq_target_register,
                    imm: eq_target as i32,
                },
                Instruction::MovImm {
                    dst: R0_IDX,
                    imm: v1 as i32,
                },
                Instruction::MovImm {
                    dst: R1_IDX,
                    imm: v2 as i32,
                },
                Instruction::MovImm {
                    dst: R2_IDX,
                    imm: v1 as i32,
                },
                Instruction::Jeq {
                    adr: eq_target_register,
                    v0: R0_IDX,
                    v1: R1_IDX,
                },
                Instruction::Jeq {
                    adr: eq_target_register,
                    v0: R0_IDX,
                    v1: R2_IDX,
                },
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
            let ne_target_register = R10_IDX;
            let ne_target = 987i64;
            let v1 = 123i64;
            let v2 = 321i64;

            let program = Program::from_instructions(&[
                Instruction::MovImm {
                    dst: ne_target_register,
                    imm: ne_target as i32,
                },
                Instruction::MovImm {
                    dst: R0_IDX,
                    imm: v1 as i32,
                },
                Instruction::MovImm {
                    dst: R1_IDX,
                    imm: v1 as i32,
                },
                Instruction::MovImm {
                    dst: R2_IDX,
                    imm: v2 as i32,
                },
                Instruction::Jne {
                    adr: ne_target_register,
                    v0: R0_IDX,
                    v1: R1_IDX,
                },
                Instruction::Jne {
                    adr: ne_target_register,
                    v0: R0_IDX,
                    v1: R2_IDX,
                },
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
            assert_eq!(program_instance.state.register_read(RP_IDX), ne_target);
        }

        #[test]
        fn exec_jgt() {
            let gt_target_register = R10_IDX;
            let gt_target = 987i64;
            let v1 = 123i64;
            let v2 = 321i64;

            let program = Program::from_instructions(&[
                Instruction::MovImm {
                    dst: gt_target_register,
                    imm: gt_target as i32,
                },
                Instruction::MovImm {
                    dst: R0_IDX,
                    imm: v1 as i32,
                },
                Instruction::MovImm {
                    dst: R1_IDX,
                    imm: v1 as i32,
                },
                Instruction::MovImm {
                    dst: R2_IDX,
                    imm: v2 as i32,
                },
                Instruction::Jgt {
                    adr: gt_target_register,
                    v0: R0_IDX,
                    v1: R1_IDX,
                },
                Instruction::Jgt {
                    adr: gt_target_register,
                    v0: R0_IDX,
                    v1: R2_IDX,
                },
                Instruction::Jgt {
                    adr: gt_target_register,
                    v0: R2_IDX,
                    v1: R0_IDX,
                },
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
            assert_eq!(program_instance.state.register_read(RP_IDX), 6);
            assert_eq!(program_instance.execute_step(), Ok(true));
            assert_eq!(program_instance.state.register_read(RP_IDX), gt_target);
        }

        #[test]
        fn exec_jge() {
            let ge_target_register = R10_IDX;
            let ge_target = 987i64;
            let v1 = 123i64;
            let v2 = 321i64;

            let program = Program::from_instructions(&[
                Instruction::MovImm {
                    dst: ge_target_register,
                    imm: ge_target as i32,
                },
                Instruction::MovImm {
                    dst: R0_IDX,
                    imm: v1 as i32,
                },
                Instruction::MovImm {
                    dst: R1_IDX,
                    imm: v1 as i32,
                },
                Instruction::MovImm {
                    dst: R2_IDX,
                    imm: v2 as i32,
                },
                Instruction::Jge {
                    adr: ge_target_register,
                    v0: R0_IDX,
                    v1: R2_IDX,
                },
                Instruction::Jge {
                    adr: ge_target_register,
                    v0: R0_IDX,
                    v1: R1_IDX,
                },
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
            assert_eq!(program_instance.state.register_read(RP_IDX), ge_target);
        }

        #[test]
        fn exec_jlt() {
            let lt_target_register = R10_IDX;
            let lt_target = 987i64;
            let v1 = 123i64;
            let v2 = 321i64;

            let program = Program::from_instructions(&[
                Instruction::MovImm {
                    dst: lt_target_register,
                    imm: lt_target as i32,
                },
                Instruction::MovImm {
                    dst: R0_IDX,
                    imm: v1 as i32,
                },
                Instruction::MovImm {
                    dst: R1_IDX,
                    imm: v1 as i32,
                },
                Instruction::MovImm {
                    dst: R2_IDX,
                    imm: v2 as i32,
                },
                Instruction::Jlt {
                    adr: lt_target_register,
                    v0: R0_IDX,
                    v1: R1_IDX,
                },
                Instruction::Jlt {
                    adr: lt_target_register,
                    v0: R2_IDX,
                    v1: R0_IDX,
                },
                Instruction::Jlt {
                    adr: lt_target_register,
                    v0: R0_IDX,
                    v1: R2_IDX,
                },
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
            assert_eq!(program_instance.state.register_read(RP_IDX), 6);
            assert_eq!(program_instance.execute_step(), Ok(true));
            assert_eq!(program_instance.state.register_read(RP_IDX), lt_target);
        }

        #[test]
        fn exec_jle() {
            let le_target_register = R10_IDX;
            let le_target = 987i64;
            let v1 = 123i64;
            let v2 = 321i64;

            let program = Program::from_instructions(&[
                Instruction::MovImm {
                    dst: le_target_register,
                    imm: le_target as i32,
                },
                Instruction::MovImm {
                    dst: R0_IDX,
                    imm: v1 as i32,
                },
                Instruction::MovImm {
                    dst: R1_IDX,
                    imm: v1 as i32,
                },
                Instruction::MovImm {
                    dst: R2_IDX,
                    imm: v2 as i32,
                },
                Instruction::Jle {
                    adr: le_target_register,
                    v0: R2_IDX,
                    v1: R0_IDX,
                },
                Instruction::Jle {
                    adr: le_target_register,
                    v0: R0_IDX,
                    v1: R1_IDX,
                },
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
            assert_eq!(program_instance.state.register_read(RP_IDX), le_target);
        }

        #[test]
        fn exec_add() {
            let v0 = 123i64;
            let v1 = 321i64;
            let vexpected = v0 + v1;

            let program = Program::from_instructions(&[
                Instruction::MovImm {
                    dst: R0_IDX,
                    imm: v0 as i32,
                },
                Instruction::MovImm {
                    dst: R1_IDX,
                    imm: v1 as i32,
                },
                Instruction::Add {
                    dst: R2_IDX,
                    v0: R0_IDX,
                    v1: R1_IDX,
                },
            ]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            for _ in 0..3 {
                assert_eq!(program_instance.execute_step(), Ok(true));
            }
            assert_eq!(program_instance.state.register_read(R2_IDX), vexpected);
        }

        #[test]
        fn exec_add_negative() {
            let v0 = 123i64;
            let v1 = -321i64;
            let vexpected = v0 + v1;

            let program = Program::from_instructions(&[
                Instruction::MovImm {
                    dst: R0_IDX,
                    imm: v0 as i32,
                },
                Instruction::MovImm {
                    dst: R1_IDX,
                    imm: v1 as i32,
                },
                Instruction::Add {
                    dst: R2_IDX,
                    v0: R0_IDX,
                    v1: R1_IDX,
                },
            ]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            for _ in 0..3 {
                assert_eq!(program_instance.execute_step(), Ok(true));
            }
            assert_eq!(program_instance.state.register_read(R2_IDX), vexpected);
        }

        #[test]
        fn exec_mul() {
            let v0 = 123i64;
            let v1 = -321i64;
            let vexpected = v0 * v1;

            let program = Program::from_instructions(&[
                Instruction::MovImm {
                    dst: R0_IDX,
                    imm: v0 as i32,
                },
                Instruction::MovImm {
                    dst: R1_IDX,
                    imm: v1 as i32,
                },
                Instruction::Mul {
                    dst: R2_IDX,
                    v0: R0_IDX,
                    v1: R1_IDX,
                },
            ]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            for _ in 0..3 {
                assert_eq!(program_instance.execute_step(), Ok(true));
            }
            assert_eq!(program_instance.state.register_read(R2_IDX), vexpected);
        }

        #[test]
        fn exec_div() {
            let v0 = 125i64;
            let v1 = 20i64;
            let vexpected = v0 / v1;

            let program = Program::from_instructions(&[
                Instruction::MovImm {
                    dst: R0_IDX,
                    imm: v0 as i32,
                },
                Instruction::MovImm {
                    dst: R1_IDX,
                    imm: v1 as i32,
                },
                Instruction::Div {
                    dst: R2_IDX,
                    v0: R0_IDX,
                    v1: R1_IDX,
                },
            ]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            for _ in 0..3 {
                assert_eq!(program_instance.execute_step(), Ok(true));
            }
            assert_eq!(program_instance.state.register_read(R2_IDX), vexpected);
        }

        #[test]
        fn exec_mod() {
            let v0 = 125i64;
            let v1 = 20i64;
            let vexpected = v0 % v1;

            let program = Program::from_instructions(&[
                Instruction::MovImm {
                    dst: R0_IDX,
                    imm: v0 as i32,
                },
                Instruction::MovImm {
                    dst: R1_IDX,
                    imm: v1 as i32,
                },
                Instruction::Mod {
                    dst: R2_IDX,
                    v0: R0_IDX,
                    v1: R1_IDX,
                },
            ]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            for _ in 0..3 {
                assert_eq!(program_instance.execute_step(), Ok(true));
            }
            assert_eq!(program_instance.state.register_read(R2_IDX), vexpected);
        }

        #[test]
        fn exec_halt() {
            let program = Program::from_instructions(&[Instruction::Halt {}]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            assert_eq!(program_instance.execute_step(), Ok(false));
            assert_eq!(program_instance.execute_step(), Err(Error::ExecutionError));
        }

        #[test]
        fn exec_noop() {
            let program = Program::from_instructions(&[
                Instruction::Noop {},
                Instruction::Noop {},
                Instruction::Noop {},
                Instruction::Noop {},
                Instruction::Halt {},
            ]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            for _ in 0..4 {
                assert_eq!(program_instance.execute_step(), Ok(true));
            }
            assert_eq!(program_instance.execute_step(), Ok(false));
        }

        #[test]
        fn exec_empty_program() {
            let program = Program::from_instructions(&[]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            assert_eq!(program_instance.execute_step(), Err(Error::ExecutionError));
        }

        #[test]
        fn exec_end_of_code() {
            let program = Program::from_instructions(&[
                Instruction::Noop {},
                Instruction::Noop {},
                Instruction::Noop {},
                Instruction::Noop {},
            ]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            for _ in 0..4 {
                assert_eq!(program_instance.execute_step(), Ok(true));
            }
            assert_eq!(program_instance.execute_step(), Err(Error::ExecutionError));
        }

        #[test]
        fn exec_reused_register() {
            let v0 = 123i64;
            let vexpected = v0 + v0;

            let program = Program::from_instructions(&[
                Instruction::MovImm {
                    dst: R0_IDX,
                    imm: v0 as i32,
                },
                Instruction::Add {
                    dst: R0_IDX,
                    v0: R0_IDX,
                    v1: R0_IDX,
                },
            ]);
            let state = ExecutionState::default();
            let mut program_instance = ProgramInstance::new(program, state);

            for _ in 0..2 {
                assert_eq!(program_instance.execute_step(), Ok(true));
            }
            assert_eq!(program_instance.state.register_read(R0_IDX), vexpected);
        }
    }
}
