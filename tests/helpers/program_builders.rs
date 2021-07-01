use vmx::auction::Side;
use vmx::vm::{Instruction, RegIdx};
use vmx::Price;

pub fn reuse_bids() -> Vec<Instruction> {
    reset_or_reuse_quotes(Side::Bid, true)
}

pub fn reset_bids() -> Vec<Instruction> {
    reset_or_reuse_quotes(Side::Bid, false)
}

pub fn reuse_asks() -> Vec<Instruction> {
    reset_or_reuse_quotes(Side::Offer, true)
}

pub fn reset_asks() -> Vec<Instruction> {
    reset_or_reuse_quotes(Side::Offer, false)
}

pub fn replace_bids(price: Price, quantity: u64) -> Vec<Instruction> {
    replace_or_modify_quotes(Side::Bid, false, price, quantity)
}

pub fn replace_asks(price: Price, quantity: u64) -> Vec<Instruction> {
    replace_or_modify_quotes(Side::Offer, false, price, quantity)
}

pub fn modify_bids(price: Price, quantity: u64) -> Vec<Instruction> {
    replace_or_modify_quotes(Side::Bid, true, price, quantity)
}

pub fn modify_asks(price: Price, quantity: u64) -> Vec<Instruction> {
    replace_or_modify_quotes(Side::Offer, true, price, quantity)
}

pub fn replace_quotes(
    bid: Price,
    bid_quantity: u64,
    ask: Price,
    ask_quantity: u64,
) -> Vec<Instruction> {
    replace_or_modify_quotes(Side::Bid, false, bid, bid_quantity)
        .into_iter()
        .chain(replace_or_modify_quotes(
            Side::Offer,
            false,
            ask,
            ask_quantity,
        ))
        .collect()
}

pub fn modify_quotes(
    bid: Price,
    bid_quantity: u64,
    ask: Price,
    ask_quantity: u64,
) -> Vec<Instruction> {
    replace_or_modify_quotes(Side::Bid, true, bid, bid_quantity)
        .into_iter()
        .chain(replace_or_modify_quotes(
            Side::Offer,
            true,
            ask,
            ask_quantity,
        ))
        .collect()
}

fn replace_or_modify_quotes(
    side: Side,
    reuse: bool,
    revision_price: Price,
    revision_quantity: u64,
) -> Vec<Instruction> {
    reset_or_reuse_quotes(side, reuse)
        .into_iter()
        .chain(insert_quote_revision(side, revision_price, revision_quantity).into_iter())
        .collect()
}

fn insert_quote_revision(
    side: Side,
    revision_price: Price,
    revision_quantity: u64,
) -> Vec<Instruction> {
    let arr = match side {
        Side::Bid => 9,
        Side::Offer => 10,
    };
    let idx = revision_price.0 as i32;
    let val = revision_quantity as i32;
    vec![
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
    ]
}

fn reset_or_reuse_quotes(side: Side, reuse: bool) -> Vec<Instruction> {
    let arr = match side {
        Side::Bid => 9,
        Side::Offer => 10,
    };
    let idx = 0;
    let val = if reuse { 0 } else { 1 };

    vec![
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
    ]
}
