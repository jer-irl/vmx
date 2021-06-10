pub mod configuration;
pub mod program;

use std::collections::HashMap;

use crate::{Price, ProductId, ParticipantId};
use configuration::AuctionConfiguration;
use crate::vm::Program;
use program::ProgramInstance;

pub struct Engine {
    configuration: AuctionConfiguration,
    product_books: HashMap<ProductId, Book>,
    programs: HashMap<ParticipantId, Program>,
}

impl Engine {
    pub fn new(configuration: AuctionConfiguration) -> Self {
        Self {
            configuration, product_books: HashMap::default(), programs: HashMap::default()
        }
    }

    pub fn add_program(&mut self, participant: ParticipantId, program: Program) {
        self.programs.insert(participant, program);
    }

    pub fn step_book(&mut self, product_id: ProductId) {
        for (participant_id, program) in &self.programs {
            self.product_books.get_mut(&product_id).expect("Unregistered product").do_participant_bidding_round(participant_id, program);
        }
    }
}

pub enum Side {
    Bid,
    Offer,
}

pub struct Book {
    levels: HashMap<Price, Level>,
}

impl Book {
    pub fn bid_bounds(&self) -> Option<(Price, Price)> {
        panic!("Unimplemented");
    }

    pub fn offer_bounds(&self) -> Option<(Price, Price)> {
        panic!("Unimplemented");
    }

    pub fn bid_quantity_at_price(&self, price: &Price) -> u64 {
        panic!("Unimplemented");
    }

    pub fn offer_quantity_at_price(&self, price: &Price) -> u64 {
        panic!("Unimplemented");
    }

    pub fn bid_bounds_for_participant(&self, participant_id: &ParticipantId) -> Option<(Price, Price)> {
        panic!("Unimplemented");
    }

    pub fn offer_bounds_for_participant(&self, participant_id: &ParticipantId) -> Option<(Price, Price)> {
        panic!("Unimplemented");
    }

    pub fn bid_quantity_at_price_for_participant(&self, price: &Price, participant_id: &ParticipantId) -> u64 {
        panic!("Unimplemented");
    }

    pub fn ask_quantity_at_price_for_participant(&self, price: &Price, participant_id: &ParticipantId) -> u64 {
        panic!("Unimplemented");
    }

    pub fn do_participant_bidding_round(&mut self, participant_id: &ParticipantId, program: &Program) {
        panic!("Unimplemented");
    }
}

struct Level {
    orders: Vec<Order>,
}

struct Order {
    participant: ParticipantId,
    side: Side,
    quantity: u64,
}
