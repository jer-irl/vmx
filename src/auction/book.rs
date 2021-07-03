use std::cmp::min;
use std::collections::HashMap;

use crate::auction::{Side, Trade};
use crate::participant::ParticipantId;
use crate::{Price, ProductId};

pub struct Book {
    pub(super) product_id: ProductId,
    pub(super) levels: HashMap<Price, Level>,
}

impl Book {
    pub fn new(product_id: ProductId) -> Self {
        Self {
            product_id,
            levels: HashMap::default(),
        }
    }

    pub fn do_matching(&mut self) -> Vec<Trade> {
        let bid_bounds = self.bid_bounds();
        let offer_bounds = self.offer_bounds();
        if bid_bounds.is_none() || offer_bounds.is_none() {
            return vec![];
        }

        let (lowest_bid, highest_bid) = bid_bounds.unwrap();
        let (lowest_offer, highest_offer) = offer_bounds.unwrap();

        let mut best_bid_level = highest_bid;
        let mut best_offer_level = lowest_offer;
        let mut result: Vec<Trade> = Vec::default();
        while best_bid_level >= best_offer_level
            && best_bid_level >= lowest_bid
            && best_offer_level <= highest_offer
        {
            let available_bid_quantity = self.bid_quantity_at_price(best_bid_level);
            let available_offer_quantity = self.offer_quantity_at_price(best_offer_level);
            let quantity_to_exhaust = min(available_bid_quantity, available_offer_quantity);
            let mut bid_quantity_to_exhaust = quantity_to_exhaust;
            let mut offer_quantity_to_exhaust = quantity_to_exhaust;
            let bid_quantity_exhaust_ratio =
                quantity_to_exhaust as f64 / available_bid_quantity as f64;
            let offer_quantity_exhaust_ratio =
                quantity_to_exhaust as f64 / available_offer_quantity as f64;

            let buy_midpoint =
                Price(((best_bid_level.0 + best_offer_level.0) as f64 / 2.0).ceil() as u64);
            let sell_midpoint =
                Price(((best_bid_level.0 + best_offer_level.0) as f64 / 2.0).floor() as u64);

            let bid_level_orders_vec = &mut self.levels.entry(best_bid_level).or_default().orders;
            for bid_order in bid_level_orders_vec
                .iter_mut()
                .filter(|o| o.side == Side::Bid)
            {
                let scaled_match =
                    (bid_order.quantity as f64 * bid_quantity_exhaust_ratio).ceil() as i64;
                let matched_quantity = min(bid_quantity_to_exhaust, scaled_match);
                result.push(Trade {
                    participant_id: bid_order.participant,
                    price: buy_midpoint,
                    product_id: self.product_id,
                    quantity: matched_quantity as u64,
                    side: Side::Bid,
                });
                bid_order.quantity -= matched_quantity;
                bid_quantity_to_exhaust -= matched_quantity;
                if bid_quantity_to_exhaust <= 0 {
                    break;
                }
            }
            bid_level_orders_vec.retain(|o| o.quantity > 0);

            let offer_level_orders_vec =
                &mut self.levels.entry(best_offer_level).or_default().orders;
            for offer_order in offer_level_orders_vec
                .iter_mut()
                .filter(|o| o.side == Side::Offer)
            {
                let scaled_match =
                    (offer_order.quantity as f64 * offer_quantity_exhaust_ratio).ceil() as i64;
                let matched_quantity = min(offer_quantity_to_exhaust, scaled_match);
                result.push(Trade {
                    participant_id: offer_order.participant,
                    price: sell_midpoint,
                    product_id: self.product_id,
                    quantity: matched_quantity as u64,
                    side: Side::Offer,
                });
                offer_order.quantity -= matched_quantity;
                offer_quantity_to_exhaust -= matched_quantity;
                if offer_quantity_to_exhaust <= 0 {
                    break;
                }
            }
            offer_level_orders_vec.retain(|o| o.quantity > 0);

            if self.bid_quantity_at_price(best_bid_level) <= 0 {
                best_bid_level = Price(best_bid_level.0 - 1);
            }
            if self.offer_quantity_at_price(best_offer_level) <= 0 {
                best_offer_level = Price(best_offer_level.0 + 1);
            }
        }
        result
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
        let book = Book {
            product_id: ProductId(0),
            levels,
        };

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
        let book = Book {
            product_id: ProductId(0),
            levels,
        };

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
        let book = Book {
            product_id: ProductId(0),
            levels,
        };

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
        let book = Book {
            product_id: ProductId(0),
            levels,
        };

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
                        quantity: 20,
                        price: Price(1),
                    },
                ],
            },
        )]
        .iter()
        .cloned()
        .collect();
        let mut book = Book {
            product_id: ProductId(0),
            levels,
        };

        let trades = book.do_matching();
        assert_eq!(trades.len(), 2);
        assert_eq!(trades.get(0).unwrap().quantity, 20);
        assert_eq!(trades.get(0).unwrap().price, Price(1));
        assert_eq!(trades.get(0).unwrap().product_id, ProductId(0));
        assert_eq!(trades.get(1).unwrap().quantity, 20);
        assert_eq!(trades.get(1).unwrap().price, Price(1));
        assert_eq!(trades.get(1).unwrap().product_id, ProductId(0));
    }

    #[test]
    fn no_match_across_spread() {
        let levels: HashMap<_, _> = [
            (
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
            ),
            (
                Price(2),
                Level {
                    orders: vec![Order {
                        participant: ParticipantId(1),
                        product_id: ProductId(0),
                        side: Side::Offer,
                        quantity: 20,
                        price: Price(2),
                    }],
                },
            ),
        ]
        .iter()
        .cloned()
        .collect();
        let mut book = Book {
            product_id: ProductId(0),
            levels,
        };

        let trades = book.do_matching();
        assert_eq!(trades.len(), 0);
    }

    #[test]
    fn match_order_different_size() {
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
                        quantity: 40,
                        price: Price(1),
                    },
                ],
            },
        )]
        .iter()
        .cloned()
        .collect();
        let mut book = Book {
            product_id: ProductId(0),
            levels,
        };

        let trades = book.do_matching();
        assert_eq!(trades.len(), 2);
        assert_eq!(trades.get(0).unwrap().quantity, 20);
        assert_eq!(trades.get(1).unwrap().quantity, 20);
    }

    #[test]
    fn match_orders_by_quantity_priority() {
        let levels: HashMap<_, _> = [(
            Price(1),
            Level {
                orders: vec![
                    Order {
                        participant: ParticipantId(0),
                        product_id: ProductId(0),
                        side: Side::Bid,
                        quantity: 10,
                        price: Price(1),
                    },
                    Order {
                        participant: ParticipantId(1),
                        product_id: ProductId(0),
                        side: Side::Offer,
                        quantity: 12,
                        price: Price(1),
                    },
                    Order {
                        participant: ParticipantId(2),
                        product_id: ProductId(0),
                        side: Side::Offer,
                        quantity: 8,
                        price: Price(1),
                    },
                ],
            },
        )]
        .iter()
        .cloned()
        .collect();
        let mut book = Book {
            product_id: ProductId(0),
            levels,
        };

        let trades = book.do_matching();
        assert_eq!(trades.len(), 3);
        let buys: Vec<_> = trades.iter().filter(|t| t.side == Side::Bid).collect();
        assert_eq!(buys.len(), 1);
        assert_eq!(buys.get(0).unwrap().quantity, 10);
        let sells: Vec<_> = trades.iter().filter(|t| t.side == Side::Offer).collect();
        assert_eq!(sells.len(), 2);
        assert!(sells.iter().find(|t| t.quantity == 6).is_some());
        assert!(sells.iter().find(|t| t.quantity == 4).is_some());
    }

    #[test]
    fn match_overlapping() {
        let levels: HashMap<_, _> = [
            (
                Price(1),
                Level {
                    orders: vec![Order {
                        participant: ParticipantId(0),
                        product_id: ProductId(0),
                        side: Side::Offer,
                        quantity: 20,
                        price: Price(1),
                    }],
                },
            ),
            (
                Price(3),
                Level {
                    orders: vec![Order {
                        participant: ParticipantId(1),
                        product_id: ProductId(0),
                        side: Side::Bid,
                        quantity: 20,
                        price: Price(3),
                    }],
                },
            ),
        ]
        .iter()
        .cloned()
        .collect();
        let mut book = Book {
            product_id: ProductId(0),
            levels,
        };

        let trades = book.do_matching();
        assert_eq!(trades.len(), 2);
        assert!(trades.iter().all(|t| t.quantity == 20));
        assert!(trades.iter().all(|t| t.price == Price(2)));
    }

    #[test]
    fn rounded_midpoint() {
        let levels: HashMap<_, _> = [
            (
                Price(1),
                Level {
                    orders: vec![Order {
                        participant: ParticipantId(0),
                        product_id: ProductId(0),
                        side: Side::Offer,
                        quantity: 20,
                        price: Price(1),
                    }],
                },
            ),
            (
                Price(4),
                Level {
                    orders: vec![Order {
                        participant: ParticipantId(1),
                        product_id: ProductId(0),
                        side: Side::Bid,
                        quantity: 20,
                        price: Price(4),
                    }],
                },
            ),
        ]
        .iter()
        .cloned()
        .collect();
        let mut book = Book {
            product_id: ProductId(0),
            levels,
        };

        let trades = book.do_matching();
        assert_eq!(
            trades.iter().find(|t| t.side == Side::Bid).unwrap().price,
            Price(3)
        );
        assert_eq!(
            trades.iter().find(|t| t.side == Side::Offer).unwrap().price,
            Price(2)
        );
    }
}
