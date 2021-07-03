use vmx::auction::Side;
use vmx::vm::{Instruction, Program, RegIdx};
use vmx::Price;

#[derive(Clone)]
pub struct ProgramBuilder {
    pending_instructions: Vec<Instruction>,
}

#[allow(dead_code)]
impl ProgramBuilder {
    pub fn new() -> Self {
        Self {
            pending_instructions: Vec::default(),
        }
    }

    pub fn build(&self) -> Program {
        Program::from_instructions(&self.pending_instructions.clone())
    }

    pub fn reuse_bids(&mut self) -> &mut Self {
        self.reset_or_reuse_quotes(Side::Bid, true)
    }

    pub fn reset_bids(&mut self) -> &mut Self {
        self.reset_or_reuse_quotes(Side::Bid, false)
    }

    pub fn reuse_asks(&mut self) -> &mut Self {
        self.reset_or_reuse_quotes(Side::Offer, true)
    }

    pub fn reset_asks(&mut self) -> &mut Self {
        self.reset_or_reuse_quotes(Side::Offer, false)
    }

    pub fn replace_bids(&mut self, price: Price, quantity: u64) -> &mut Self {
        self.replace_or_modify_quotes(Side::Bid, false, price, quantity)
    }

    pub fn replace_asks(&mut self, price: Price, quantity: u64) -> &mut Self {
        self.replace_or_modify_quotes(Side::Offer, false, price, quantity)
    }

    pub fn modify_bids(&mut self, price: Price, quantity: u64) -> &mut Self {
        self.replace_or_modify_quotes(Side::Bid, true, price, quantity)
    }

    pub fn modify_asks(&mut self, price: Price, quantity: u64) -> &mut Self {
        self.replace_or_modify_quotes(Side::Offer, true, price, quantity)
    }

    pub fn replace_quotes(
        &mut self,
        bid: Price,
        bid_quantity: u64,
        ask: Price,
        ask_quantity: u64,
    ) -> &mut Self {
        self.replace_or_modify_quotes(Side::Bid, false, bid, bid_quantity)
            .replace_or_modify_quotes(Side::Offer, false, ask, ask_quantity)
    }

    pub fn modify_quotes(
        &mut self,
        bid: Price,
        bid_quantity: u64,
        ask: Price,
        ask_quantity: u64,
    ) -> &mut Self {
        self.replace_or_modify_quotes(Side::Bid, true, bid, bid_quantity)
            .replace_or_modify_quotes(Side::Offer, true, ask, ask_quantity)
    }

    fn replace_or_modify_quotes(
        &mut self,
        side: Side,
        reuse: bool,
        revision_price: Price,
        revision_quantity: u64,
    ) -> &mut Self {
        self.reset_or_reuse_quotes(side, reuse)
            .insert_quote_revision(side, revision_price, revision_quantity)
    }

    fn insert_quote_revision(
        &mut self,
        side: Side,
        revision_price: Price,
        revision_quantity: u64,
    ) -> &mut Self {
        let arr = match side {
            Side::Bid => 9,
            Side::Offer => 10,
        };
        let idx = revision_price.0 as i32;
        let val = revision_quantity as i32;
        self.pending_instructions.extend([
            Instruction::MovImm {
                dst: RegIdx(0),
                imm: arr,
            },
            Instruction::MovImm {
                dst: RegIdx(1),
                imm: idx,
            },
            Instruction::MovImm {
                dst: RegIdx(2),
                imm: val,
            },
            Instruction::ArrIns {
                arr: RegIdx(0),
                idx: RegIdx(1),
                val: RegIdx(2),
            },
        ]);
        self
    }

    fn reset_or_reuse_quotes(&mut self, side: Side, reuse: bool) -> &mut Self {
        let arr = match side {
            Side::Bid => 9,
            Side::Offer => 10,
        };
        let idx = 0;
        let val = if reuse { 0 } else { 1 };

        self.pending_instructions.extend([
            Instruction::MovImm {
                dst: RegIdx(0),
                imm: arr,
            },
            Instruction::MovImm {
                dst: RegIdx(1),
                imm: idx,
            },
            Instruction::MovImm {
                dst: RegIdx(2),
                imm: val,
            },
            Instruction::ArrIns {
                arr: RegIdx(0),
                idx: RegIdx(1),
                val: RegIdx(2),
            },
        ]);
        self
    }
}
