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

#[derive(Default, Clone)]
pub struct ParticipantParameters {
    values: HashMap<u64, i64>,
}

#[derive(Default)]
struct ParticipantRecord {
    interested_product_programs: HashMap<ProductId, Program>,
    interested_product_parameters: HashMap<ProductId, ParticipantParameters>,
}

pub struct Trade {
    pub product_id: ProductId,
    pub participant_id: ParticipantId,
    pub side: Side,
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
        participant_id: ParticipantId,
        directive: &ClientDirective,
    ) {
        match directive {
            ClientDirective::Join {} => {
                self.participants
                    .insert(participant_id, ParticipantRecord::default())
                    .and_then::<(), _>(|_existing_record| panic!("Multiple joins (TODO)"));
            }
            ClientDirective::Leave {} => {
                self.remove_participant_orders(participant_id);
                self.participants.remove(&participant_id).expect("TODO");
            }
            ClientDirective::SubmitProgram {
                product_id,
                program,
            } => {
                self.product_books
                    .entry(*product_id)
                    .or_insert(Book::new(*product_id));
                self.participants
                    .get_mut(&participant_id)
                    .expect("TODO")
                    .interested_product_programs
                    .insert(*product_id, program.clone());
            }
            ClientDirective::UpdateParameter {
                product_id,
                param_idx,
                value,
            } => {
                self.participants
                    .get_mut(&participant_id)
                    .expect("TODO")
                    .interested_product_parameters
                    .get_mut(product_id)
                    .expect("TODO")
                    .values
                    .insert(*param_idx, *value);
            }
        }
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

    pub fn step_all_books_one_auction(&mut self) {
        for _ in 0..self.configuration.num_bidding_rounds {
            for product_id in self.product_books.keys().cloned().collect::<Vec<_>>() {
                self.step_book_one_round(product_id);
            }
        }
    }

    fn step_book_one_round(&mut self, product_id: ProductId) {
        let prev_book = self
            .product_books
            .remove(&product_id)
            .expect("Missing product ID, TODO");
        let mut result_book = Book::new(product_id);

        let interested_participant_ids = self.participants.iter().filter_map(|(id, record)| {
            if record.interested_product_programs.contains_key(&product_id) {
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
            .interested_product_programs
            .get(&product_id)
            .expect("TODO");
        let participant_parameters = participant_record
            .interested_product_parameters
            .get(&product_id)
            .map(Clone::clone)
            .unwrap_or_default();
        let mut program_instance = ProgramInstance::new(
            participant_program,
            prev_book,
            participant_id,
            &participant_parameters,
        );
        program_instance.execute();
        program_instance.write_result_into_book(prev_book, result_book, participant_id);
    }

    fn remove_participant_orders(&mut self, participant_id: ParticipantId) {
        for (_, book) in &mut self.product_books {
            for (_, level) in &mut book.levels {
                level.orders.retain(|o| o.participant != participant_id)
            }
        }
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
