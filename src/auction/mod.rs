pub mod configuration;

use std::collections::HashMap;

use crate::{Price, ProductId, ParticipantId};
use configuration::AuctionConfiguration;

pub struct Engine {
    configuration: AuctionConfiguration,
    product_books: HashMap<ProductId, Book>,
}

impl Engine {
    pub fn new(configuration: AuctionConfiguration) -> Self {
        // TODO 
        Self { configuration, product_books: HashMap::default() }
    }
}

pub enum Side {
    Bid,
    Offer,
}

struct Book {
    levels: HashMap<Price, Level>,
}

struct Level {
    orders: Vec<Order>,
}

struct Order {
    participant: ParticipantId,
    side: Side,
    quantity: u64,
}
