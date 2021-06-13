pub mod configuration;
pub mod bidding_program;

use std::collections::HashMap;

use crate::{Price, ProductId, ParticipantId};
use configuration::AuctionConfiguration;
use crate::vm::Program;
use bidding_program::ProgramInstance;

struct ParticipantRecord {
    interested_products: HashMap<ProductId, Program>,
}

pub struct Engine {
    configuration: AuctionConfiguration,
    product_books: HashMap<ProductId, Book>,
    participants: HashMap<ParticipantId, ParticipantRecord>,
}

impl Engine {
    pub fn new(configuration: AuctionConfiguration) -> Self {
        Self {
            configuration, product_books: HashMap::default(), participants: HashMap::default()
        }
    }

    fn step_all_books(&mut self) {
        for product_id in self.product_books.keys().cloned().collect::<Vec<_>>() {
            self.step_book(product_id);
        }
    }

    fn step_book(&mut self, product_id: ProductId) {
        let prev_book = self.product_books.remove(&product_id).expect("Missing product ID, TODO");
        let mut result_book = Book::default();

        let interested_participant_ids = self.participants.iter().filter_map(|(id, record)| {
            if record.interested_products.contains_key(&product_id) {
                Some(id)
            } else { 
                None 
            }
        });
        for &participant_id in interested_participant_ids {
            self.apply_participant_program_to_book(participant_id, product_id, &prev_book, &mut result_book);

        }
        self.product_books.insert(product_id, result_book);
    }

    fn apply_participant_program_to_book(&self, participant_id: ParticipantId, product_id: ProductId, prev_book: &Book, result_book: &mut Book) {
        let participant_record = self.participants.get(&participant_id).expect("TODO");
        let participant_program = participant_record.interested_products.get(&product_id).expect("TODO");
        let mut program_instance = ProgramInstance::new(participant_program, prev_book, participant_id);
        program_instance.execute();
        program_instance.write_result_into_book(prev_book, result_book, participant_id);
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Side {
    Bid,
    Offer,
}

#[derive(Default)]
pub struct Book {
    levels: HashMap<Price, Level>,
}

impl Book {
    pub fn insert_order(&mut self, order: Order) {
        self.levels
            .entry(order.price)
            .or_default()
            .orders
            .push(order);
    }

    fn order_bounds<F>(&self, predicate: F) -> Option<(Price, Price)>
    where 
        F: Fn(&&Order) -> bool,
    {
        let mut orders_iter = self.levels.values().flat_map(|level| &level.orders).filter(&predicate);
        let min_max_order = (orders_iter.next(), orders_iter.last());
        if let (Some(&Order { price: min_price, .. }), Some(&Order { price: max_price, .. })) = min_max_order {
            Some((min_price, max_price))
        } else {
            None
        }
    }

    fn quantity_at_price<F>(&self, price: Price, predicate: F) -> u64
    where
        F: Fn(&&Order) -> bool,
    {
        self.levels
            .get(&price)
            .and_then(|level| Some(level.orders.iter().filter(&predicate).fold(0, |acc, order| acc + order.quantity)))
            .unwrap_or(0)
    }

    pub fn bid_bounds(&self) -> Option<(Price, Price)> {
        self.order_bounds(|order| order.side == Side::Bid)
    }

    pub fn offer_bounds(&self) -> Option<(Price, Price)> {
        self.order_bounds(|order| order.side == Side::Offer)
    }

    pub fn bid_quantity_at_price(&self, price: Price) -> u64 {
        self.quantity_at_price(price, |order| order.side == Side::Bid)
    }

    pub fn offer_quantity_at_price(&self, price: Price) -> u64 {
        self.quantity_at_price(price, |order| order.side == Side::Offer)
    }

    pub fn bid_bounds_for_participant(&self, participant_id: ParticipantId) -> Option<(Price, Price)> {
        self.order_bounds(|order| order.side == Side::Bid && order.participant == participant_id)
    }

    pub fn offer_bounds_for_participant(&self, participant_id: ParticipantId) -> Option<(Price, Price)> {
        self.order_bounds(|order| order.side == Side::Offer && order.participant == participant_id)
    }

    pub fn bid_quantity_at_price_for_participant(&self, price: Price, participant_id: ParticipantId) -> u64 {
        self.quantity_at_price(price, |order| order.side == Side::Bid && order.participant == participant_id)
    }

    pub fn offer_quantity_at_price_for_participant(&self, price: Price, participant_id: ParticipantId) -> u64 {
        self.quantity_at_price(price, |order| order.side == Side::Offer && order.participant == participant_id)
    }
}

#[derive(Clone, Default)]
struct Level {
    orders: Vec<Order>,
}

#[derive(Clone)]
pub struct Order {
    participant: ParticipantId,
    side: Side,
    quantity: u64,
    price: Price,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod book {
        use super::*;

        #[test]
        fn get_order_bounds_single_order() {
            let levels: HashMap<_, _> = [
                (
                    Price(1), 
                    Level {
                        orders: vec![ 
                            Order {
                                participant: ParticipantId(0),
                                side: Side::Bid,
                                quantity: 20,
                                price: Price(1),
                            },
                        ] 
                    }
                ),
            ].iter().cloned().collect();
            let book = Book { levels };

            assert_eq!(book.bid_bounds(), Some((Price(1), Price(1))));
            assert_eq!(book.offer_bounds(), None);
            assert_eq!(book.bid_bounds_for_participant(ParticipantId(0)), Some((Price(1), Price(1))));
            assert_eq!(book.bid_bounds_for_participant(ParticipantId(1)), None);
            assert_eq!(book.offer_bounds_for_participant(ParticipantId(0)), None);
            assert_eq!(book.offer_bounds_for_participant(ParticipantId(1)), None);
        }

        #[test]
        fn get_order_bounds_multiple_orders_one_level() {
            let levels: HashMap<_, _> = [
                (
                    Price(1), 
                    Level {
                        orders: vec![ 
                            Order {
                                participant: ParticipantId(0),
                                side: Side::Bid,
                                quantity: 20,
                                price: Price(1),
                            },
                            Order{
                                participant: ParticipantId(1),
                                side: Side::Offer,
                                quantity: 42,
                                price: Price(1),
                            }
                        ] 
                    }
                ),
            ].iter().cloned().collect();
            let book = Book { levels };

            assert_eq!(book.bid_bounds(), Some((Price(1), Price(1))));
            assert_eq!(book.offer_bounds(), Some((Price(1), Price(1))));
            assert_eq!(book.bid_bounds_for_participant(ParticipantId(0)), Some((Price(1), Price(1))));
            assert_eq!(book.bid_bounds_for_participant(ParticipantId(1)), None);
            assert_eq!(book.offer_bounds_for_participant(ParticipantId(0)), None);
            assert_eq!(book.offer_bounds_for_participant(ParticipantId(1)), Some((Price(1), Price(1))));
        }

        #[test]
        fn get_order_bounds_multiple_orders_multiple_levels() {
            let levels: HashMap<_, _> = [
                (
                    Price(1), 
                    Level {
                        orders: vec![ 
                            Order {
                                participant: ParticipantId(0),
                                side: Side::Bid,
                                quantity: 20,
                                price: Price(1),
                            },
                            Order{
                                participant: ParticipantId(1),
                                side: Side::Offer,
                                quantity: 42,
                                price: Price(1),
                            }
                        ] 
                    }
                ),
                (
                    Price(23),
                    Level {
                        orders: vec![
                            Order {
                                participant: ParticipantId(0),
                                side: Side::Bid,
                                quantity: 23,
                                price: Price(23),
                            }
                        ]
                    }
                ),
            ].iter().cloned().collect();
            let book = Book { levels };

            assert_eq!(book.bid_bounds(), Some((Price(1), Price(23))));
            assert_eq!(book.offer_bounds(), Some((Price(1), Price(1))));
            assert_eq!(book.bid_bounds_for_participant(ParticipantId(0)), Some((Price(1), Price(23))));
            assert_eq!(book.bid_bounds_for_participant(ParticipantId(1)), None);
            assert_eq!(book.offer_bounds_for_participant(ParticipantId(0)), None);
            assert_eq!(book.offer_bounds_for_participant(ParticipantId(1)), Some((Price(1), Price(1))));
        }

        #[test]
        fn get_price_level_quantities() {
            let levels: HashMap<_, _> = [
                (
                    Price(1), 
                    Level {
                        orders: vec![ 
                            Order {
                                participant: ParticipantId(0),
                                side: Side::Bid,
                                quantity: 20,
                                price: Price(1),
                            },
                            Order {
                                participant: ParticipantId(0),
                                side: Side::Bid,
                                quantity: 1,
                                price: Price(1),
                            },
                            Order{
                                participant: ParticipantId(1),
                                side: Side::Offer,
                                quantity: 42,
                                price: Price(1),
                            },
                            Order {
                                participant: ParticipantId(2),
                                side: Side::Bid,
                                quantity: 99,
                                price: Price(1),
                            }
                        ] 
                    }
                ),
                (
                    Price(23),
                    Level {
                        orders: vec![
                            Order {
                                participant: ParticipantId(0),
                                side: Side::Bid,
                                quantity: 23,
                                price: Price(23),
                            }
                        ]
                    }
                ),
            ].iter().cloned().collect();
            let book = Book { levels };

            assert_eq!(book.bid_quantity_at_price(Price(1)), 20 + 1 + 99);
            assert_eq!(book.bid_quantity_at_price(Price(23)), 23);
            assert_eq!(book.bid_quantity_at_price(Price(42)), 0);

            assert_eq!(book.offer_quantity_at_price(Price(1)), 42);
            assert_eq!(book.offer_quantity_at_price(Price(23)), 0);
            assert_eq!(book.offer_quantity_at_price(Price(42)), 0);

            assert_eq!(book.bid_quantity_at_price_for_participant(Price(1), ParticipantId(0)), 20 + 1);
            assert_eq!(book.bid_quantity_at_price_for_participant(Price(1), ParticipantId(1)), 0);
            assert_eq!(book.bid_quantity_at_price_for_participant(Price(1), ParticipantId(2)), 99);

            assert_eq!(book.bid_quantity_at_price_for_participant(Price(23), ParticipantId(0)), 23);
            assert_eq!(book.bid_quantity_at_price_for_participant(Price(23), ParticipantId(1)), 0);
            assert_eq!(book.bid_quantity_at_price_for_participant(Price(23), ParticipantId(2)), 0);

            assert_eq!(book.bid_quantity_at_price_for_participant(Price(42), ParticipantId(0)), 0);
            assert_eq!(book.bid_quantity_at_price_for_participant(Price(42), ParticipantId(1)), 0);
            assert_eq!(book.bid_quantity_at_price_for_participant(Price(42), ParticipantId(2)), 0);

            assert_eq!(book.offer_quantity_at_price_for_participant(Price(1), ParticipantId(0)), 0);
            assert_eq!(book.offer_quantity_at_price_for_participant(Price(1), ParticipantId(1)), 42);
            assert_eq!(book.offer_quantity_at_price_for_participant(Price(1), ParticipantId(2)), 0);

            assert_eq!(book.offer_quantity_at_price_for_participant(Price(23), ParticipantId(0)), 0);
            assert_eq!(book.offer_quantity_at_price_for_participant(Price(23), ParticipantId(1)), 0);
            assert_eq!(book.offer_quantity_at_price_for_participant(Price(23), ParticipantId(2)), 0);

            assert_eq!(book.offer_quantity_at_price_for_participant(Price(42), ParticipantId(0)), 0);
            assert_eq!(book.offer_quantity_at_price_for_participant(Price(42), ParticipantId(1)), 0);
            assert_eq!(book.offer_quantity_at_price_for_participant(Price(42), ParticipantId(2)), 0);
        }
    }

    mod engine {
        use super::*;

        #[test]
        fn step_all_books() {
            panic!("Unimplemented");
        }
    }
}
