use std::collections::HashMap;

use crate::auction::{Side, Trade};
use crate::participant::ParticipantId;
use crate::{Price, ProductId};

#[derive(Default)]
pub struct Book {
    pub(super) levels: HashMap<Price, Level>,
}

impl Book {
    pub fn do_matching(&mut self) -> Vec<Trade> {
        panic!("Unimplemented");
    }

    pub fn update_or_insert_order(&mut self, order: Order) {
        let found_existing = self.levels.get_mut(&order.price).and_then(|level| {
            level
                .orders
                .iter_mut()
                .find(|possible_match| possible_match.participant == order.participant)
        });
        if let Some(existing_order) = found_existing {
            existing_order.quantity += order.quantity;
        } else {
            self.insert_order(order);
        }
    }

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
        let orders_iter = self
            .levels
            .values()
            .flat_map(|level| &level.orders)
            .filter(&predicate)
            .filter(|order| order.price != Price(0));
        let min_max_order = (
            orders_iter.clone().min_by_key(|o| o.price),
            orders_iter.clone().max_by_key(|o| o.price),
        );
        if let (
            Some(&Order {
                price: min_price, ..
            }),
            Some(&Order {
                price: max_price, ..
            }),
        ) = min_max_order
        {
            Some((min_price, max_price))
        } else {
            None
        }
    }

    fn quantity_at_price<F>(&self, price: Price, predicate: F) -> i64
    where
        F: Fn(&&Order) -> bool,
    {
        self.levels
            .get(&price)
            .map(|level| {
                level
                    .orders
                    .iter()
                    .filter(&predicate)
                    .map(|o| o.quantity)
                    .sum()
            })
            .unwrap_or(0)
    }

    pub fn bid_bounds(&self) -> Option<(Price, Price)> {
        self.order_bounds(|order| order.side == Side::Bid)
    }

    pub fn offer_bounds(&self) -> Option<(Price, Price)> {
        self.order_bounds(|order| order.side == Side::Offer)
    }

    pub fn bid_quantity_at_price(&self, price: Price) -> i64 {
        self.quantity_at_price(price, |order| order.side == Side::Bid)
    }

    pub fn offer_quantity_at_price(&self, price: Price) -> i64 {
        self.quantity_at_price(price, |order| order.side == Side::Offer)
    }

    pub fn bid_bounds_for_participant(
        &self,
        participant_id: ParticipantId,
    ) -> Option<(Price, Price)> {
        self.order_bounds(|order| order.side == Side::Bid && order.participant == participant_id)
    }

    pub fn offer_bounds_for_participant(
        &self,
        participant_id: ParticipantId,
    ) -> Option<(Price, Price)> {
        self.order_bounds(|order| order.side == Side::Offer && order.participant == participant_id)
    }

    pub fn bid_quantity_at_price_for_participant(
        &self,
        price: Price,
        participant_id: ParticipantId,
    ) -> i64 {
        self.quantity_at_price(price, |order| {
            order.side == Side::Bid && order.participant == participant_id
        })
    }

    pub fn offer_quantity_at_price_for_participant(
        &self,
        price: Price,
        participant_id: ParticipantId,
    ) -> i64 {
        self.quantity_at_price(price, |order| {
            order.side == Side::Offer && order.participant == participant_id
        })
    }
}

#[derive(Clone, Default)]
pub(super) struct Level {
    pub(super) orders: Vec<Order>,
}

#[derive(Clone)]
pub struct Order {
    pub(super) participant: ParticipantId,
    pub(super) product_id: ProductId,
    pub(super) side: Side,
    pub(super) quantity: i64,
    pub(super) price: Price,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_order_bounds_single_order() {
        let levels: HashMap<_, _> = [(
            Price(1),
            Level {
                orders: vec![Order {
                    participant: ParticipantId(0),
                    product_id: ProductId(0),
                    side: Side::Bid,
                    quantity: 20,
                    price: Price(1),
                }],
            },
        )]
        .iter()
        .cloned()
        .collect();
        let book = Book { levels };

        assert_eq!(book.bid_bounds(), Some((Price(1), Price(1))));
        assert_eq!(book.offer_bounds(), None);
        assert_eq!(
            book.bid_bounds_for_participant(ParticipantId(0)),
            Some((Price(1), Price(1)))
        );
        assert_eq!(book.bid_bounds_for_participant(ParticipantId(1)), None);
        assert_eq!(book.offer_bounds_for_participant(ParticipantId(0)), None);
        assert_eq!(book.offer_bounds_for_participant(ParticipantId(1)), None);
    }

    #[test]
    fn get_order_bounds_multiple_orders_one_level() {
        let levels: HashMap<_, _> = [(
            Price(1),
            Level {
                orders: vec![
                    Order {
                        participant: ParticipantId(0),
                        product_id: ProductId(0),
                        side: Side::Bid,
                        quantity: 20,
                        price: Price(1),
                    },
                    Order {
                        participant: ParticipantId(1),
                        product_id: ProductId(0),
                        side: Side::Offer,
                        quantity: 42,
                        price: Price(1),
                    },
                ],
            },
        )]
        .iter()
        .cloned()
        .collect();
        let book = Book { levels };

        assert_eq!(book.bid_bounds(), Some((Price(1), Price(1))));
        assert_eq!(book.offer_bounds(), Some((Price(1), Price(1))));
        assert_eq!(
            book.bid_bounds_for_participant(ParticipantId(0)),
            Some((Price(1), Price(1)))
        );
        assert_eq!(book.bid_bounds_for_participant(ParticipantId(1)), None);
        assert_eq!(book.offer_bounds_for_participant(ParticipantId(0)), None);
        assert_eq!(
            book.offer_bounds_for_participant(ParticipantId(1)),
            Some((Price(1), Price(1)))
        );
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
                            product_id: ProductId(0),
                            side: Side::Bid,
                            quantity: 20,
                            price: Price(1),
                        },
                        Order {
                            participant: ParticipantId(1),
                            product_id: ProductId(0),
                            side: Side::Offer,
                            quantity: 42,
                            price: Price(1),
                        },
                    ],
                },
            ),
            (
                Price(23),
                Level {
                    orders: vec![Order {
                        participant: ParticipantId(0),
                        product_id: ProductId(0),
                        side: Side::Bid,
                        quantity: 23,
                        price: Price(23),
                    }],
                },
            ),
        ]
        .iter()
        .cloned()
        .collect();
        let book = Book { levels };

        assert_eq!(book.bid_bounds(), Some((Price(1), Price(23))));
        assert_eq!(book.offer_bounds(), Some((Price(1), Price(1))));
        assert_eq!(
            book.bid_bounds_for_participant(ParticipantId(0)),
            Some((Price(1), Price(23)))
        );
        assert_eq!(book.bid_bounds_for_participant(ParticipantId(1)), None);
        assert_eq!(book.offer_bounds_for_participant(ParticipantId(0)), None);
        assert_eq!(
            book.offer_bounds_for_participant(ParticipantId(1)),
            Some((Price(1), Price(1)))
        );
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
                            product_id: ProductId(0),
                            side: Side::Bid,
                            quantity: 20,
                            price: Price(1),
                        },
                        Order {
                            participant: ParticipantId(0),
                            product_id: ProductId(0),
                            side: Side::Bid,
                            quantity: 1,
                            price: Price(1),
                        },
                        Order {
                            participant: ParticipantId(1),
                            product_id: ProductId(0),
                            side: Side::Offer,
                            quantity: 42,
                            price: Price(1),
                        },
                        Order {
                            participant: ParticipantId(2),
                            product_id: ProductId(0),
                            side: Side::Bid,
                            quantity: 99,
                            price: Price(1),
                        },
                    ],
                },
            ),
            (
                Price(23),
                Level {
                    orders: vec![Order {
                        participant: ParticipantId(0),
                        product_id: ProductId(0),
                        side: Side::Bid,
                        quantity: 23,
                        price: Price(23),
                    }],
                },
            ),
        ]
        .iter()
        .cloned()
        .collect();
        let book = Book { levels };

        assert_eq!(book.bid_quantity_at_price(Price(1)), 20 + 1 + 99);
        assert_eq!(book.bid_quantity_at_price(Price(23)), 23);
        assert_eq!(book.bid_quantity_at_price(Price(42)), 0);

        assert_eq!(book.offer_quantity_at_price(Price(1)), 42);
        assert_eq!(book.offer_quantity_at_price(Price(23)), 0);
        assert_eq!(book.offer_quantity_at_price(Price(42)), 0);

        assert_eq!(
            book.bid_quantity_at_price_for_participant(Price(1), ParticipantId(0)),
            20 + 1
        );
        assert_eq!(
            book.bid_quantity_at_price_for_participant(Price(1), ParticipantId(1)),
            0
        );
        assert_eq!(
            book.bid_quantity_at_price_for_participant(Price(1), ParticipantId(2)),
            99
        );

        assert_eq!(
            book.bid_quantity_at_price_for_participant(Price(23), ParticipantId(0)),
            23
        );
        assert_eq!(
            book.bid_quantity_at_price_for_participant(Price(23), ParticipantId(1)),
            0
        );
        assert_eq!(
            book.bid_quantity_at_price_for_participant(Price(23), ParticipantId(2)),
            0
        );

        assert_eq!(
            book.bid_quantity_at_price_for_participant(Price(42), ParticipantId(0)),
            0
        );
        assert_eq!(
            book.bid_quantity_at_price_for_participant(Price(42), ParticipantId(1)),
            0
        );
        assert_eq!(
            book.bid_quantity_at_price_for_participant(Price(42), ParticipantId(2)),
            0
        );

        assert_eq!(
            book.offer_quantity_at_price_for_participant(Price(1), ParticipantId(0)),
            0
        );
        assert_eq!(
            book.offer_quantity_at_price_for_participant(Price(1), ParticipantId(1)),
            42
        );
        assert_eq!(
            book.offer_quantity_at_price_for_participant(Price(1), ParticipantId(2)),
            0
        );

        assert_eq!(
            book.offer_quantity_at_price_for_participant(Price(23), ParticipantId(0)),
            0
        );
        assert_eq!(
            book.offer_quantity_at_price_for_participant(Price(23), ParticipantId(1)),
            0
        );
        assert_eq!(
            book.offer_quantity_at_price_for_participant(Price(23), ParticipantId(2)),
            0
        );

        assert_eq!(
            book.offer_quantity_at_price_for_participant(Price(42), ParticipantId(0)),
            0
        );
        assert_eq!(
            book.offer_quantity_at_price_for_participant(Price(42), ParticipantId(1)),
            0
        );
        assert_eq!(
            book.offer_quantity_at_price_for_participant(Price(42), ParticipantId(2)),
            0
        );
    }

    #[test]
    fn match_order_same_size() {
        panic!("Unimplemented");
    }

    #[test]
    fn no_match_across_spread() {
        panic!("Unimplemented");
    }

    #[test]
    fn match_order_different_size() {
        panic!("Unimplemented");
    }

    #[test]
    fn match_orders_by_priority() {
        panic!("Unimplemented");
    }

    #[test]
    fn match_many_overlapping() {
        panic!("Unimplemented");
    }

    #[test]
    fn match_vcg() {
        panic!("Unimplemented");
    }
}
