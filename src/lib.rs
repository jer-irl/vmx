pub mod auction;
pub mod vm;

use num_derive::NumOps;

#[derive(Debug, NumOps, Clone, Copy, PartialOrd, PartialEq, Hash, Eq, Ord)]
pub struct Price(u64);

impl Into<i64> for Price {
    fn into(self) -> i64 {
        self.0 as i64
    }
}

impl Into<u64> for Price {
    fn into(self) -> u64 {
        self.0
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct ParticipantId(u64);

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct ProductId(u64);
