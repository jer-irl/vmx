pub mod auction;
pub mod vm;

use num_derive::NumOps;

#[derive(Debug, NumOps)]
pub struct Price(u64);

pub struct ParticipantId(u64);

pub struct ProductId(u64);
