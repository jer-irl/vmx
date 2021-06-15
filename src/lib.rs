#![warn(clippy::all)]

pub mod auction;
pub mod exchange;
pub mod protocol;
pub mod server;
pub mod vm;

use num_derive::NumOps;

#[derive(Debug, NumOps, Clone, Copy, PartialOrd, PartialEq, Hash, Eq, Ord)]
pub struct Price(u64);

impl From<Price> for i64 {
    fn from(p: Price) -> Self {
        p.0 as i64
    }
}

impl From<Price> for u64 {
    fn from(p: Price) -> Self {
        p.0
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct ParticipantId(u64);

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct ProductId(u64);
