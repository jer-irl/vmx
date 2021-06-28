mod bidding_program;
mod book;
mod configuration;

use std::collections::HashMap;

use crate::participant::ParticipantId;
use crate::protocol::ClientDirective;
use crate::vm::Program;
use crate::{Price, ProductId};
pub use bidding_program::ProgramInstance;
pub use book::{Book, Order};
pub use configuration::AuctionConfiguration;

struct ParticipantRecord {
    interested_products: HashMap<ProductId, Program>,
}

pub struct Trade {
    pub product_id: ProductId,
    pub buyer: ParticipantId,
    pub seller: ParticipantId,
    pub price: Price,
    pub quantity: u64,
}

pub struct Engine {
    configuration: AuctionConfiguration,
    product_books: HashMap<ProductId, Book>,
    participants: HashMap<ParticipantId, ParticipantRecord>,
}

impl Engine {
    pub fn new(configuration: AuctionConfiguration) -> Self {
        Self {
            configuration,
            product_books: HashMap::default(),
            participants: HashMap::default(),
        }
    }

    pub fn apply_participant_directive(
        &mut self,
        _participant_id: ParticipantId,
        _directive: &ClientDirective,
    ) {
        todo!();
    }

    pub fn config(&self) -> &AuctionConfiguration {
        &self.configuration
    }

    pub fn match_all_books(&mut self) -> Vec<Trade> {
        self.product_books
            .iter_mut()
            .map(|(_product_id, book)| book.do_matching())
            .flatten()
            .collect()
    }

    pub fn step_all_books(&mut self) {
        for product_id in self.product_books.keys().cloned().collect::<Vec<_>>() {
            self.step_book(product_id);
        }
    }

    fn step_book(&mut self, product_id: ProductId) {
        let prev_book = self
            .product_books
            .remove(&product_id)
            .expect("Missing product ID, TODO");
        let mut result_book = Book::default();

        let interested_participant_ids = self.participants.iter().filter_map(|(id, record)| {
            if record.interested_products.contains_key(&product_id) {
                Some(id)
            } else {
                None
            }
        });
        for &participant_id in interested_participant_ids {
            self.apply_participant_program_to_book(
                participant_id,
                product_id,
                &prev_book,
                &mut result_book,
            );
        }
        self.product_books.insert(product_id, result_book);
    }

    fn apply_participant_program_to_book(
        &self,
        participant_id: ParticipantId,
        product_id: ProductId,
        prev_book: &Book,
        result_book: &mut Book,
    ) {
        let participant_record = self.participants.get(&participant_id).expect("TODO");
        let participant_program = participant_record
            .interested_products
            .get(&product_id)
            .expect("TODO");
        let mut program_instance =
            ProgramInstance::new(participant_program, prev_book, participant_id);
        program_instance.execute();
        program_instance.write_result_into_book(prev_book, result_book, participant_id);
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Side {
    Bid,
    Offer,
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
