use super::{Book, Order, ProductId, Side};
use crate::participant::ParticipantId;
use crate::{vm, Price};

/// Input
/// ```{text}
/// arr1[0]: min bid price or 0 if none
/// arr1[1]: max bid price or 0 if none
/// arr2[price]: #bids at price
///
/// arr3[0]: min offer price or 0 if none
/// arr3[1]: max offer price or 0 if none
/// arr4[price]: #offers at price
///
/// arr5[0]: my min bid price or 0 if none
/// arr5[0]: my max bid price or 0 if none
/// arr6[price]: #my bids at price
///
/// arr7[0]: my min offer price or 0 if none
/// arr7[0]: my max offer price or 0 if none
/// arr8[price]: #my offers at price
/// ```
///
/// Output
/// ```{text}
/// arr9[0]: 0 if reusing old bids, 1 if erasing old bids
/// arr9[price]: #bids to add or subtract at price (negative result is error)
/// arr10[0]: 0 if reusing old offers, 1 if erasing old offers
/// arr10[price]: #offers to add or subtract at price (negative result is error)
/// ```
pub struct ProgramInstance {
    vm_program_instance: vm::ProgramInstance,
}

impl ProgramInstance {
    pub fn new(program: &vm::Program, book: &Book, participant: ParticipantId) -> Self {
        let mut state = vm::ExecutionState::default();

        if let Some((lowest_bid, highest_bid)) = book.bid_bounds() {
            state.array_insert(1, 0, lowest_bid.into());
            state.array_insert(1, 1, highest_bid.into());
            for price in (lowest_bid.into()..=highest_bid.into()).map(|idx| Price(idx)) {
                state.array_insert(2, price.into(), book.bid_quantity_at_price(price) as i64);
            }
        }

        if let Some((lowest_offer, highest_offer)) = book.offer_bounds() {
            state.array_insert(3, 0, lowest_offer.into());
            state.array_insert(3, 1, highest_offer.into());
            for price in (lowest_offer.into()..=highest_offer.into()).map(|idx| Price(idx)) {
                state.array_insert(4, price.into(), book.offer_quantity_at_price(price) as i64);
            }
        }

        if let Some((participant_lowest_bid, participant_highest_bid)) =
            book.bid_bounds_for_participant(participant)
        {
            state.array_insert(5, 0, participant_lowest_bid.into());
            state.array_insert(5, 1, participant_highest_bid.into());
            for price in (participant_lowest_bid.into()..=participant_highest_bid.into())
                .map(|idx| Price(idx))
            {
                state.array_insert(
                    6,
                    price.into(),
                    book.bid_quantity_at_price_for_participant(price, participant) as i64,
                );
            }
        }

        if let Some((participant_lowest_offer, participant_highest_offer)) =
            book.offer_bounds_for_participant(participant)
        {
            state.array_insert(5, 0, participant_lowest_offer.into());
            state.array_insert(5, 1, participant_highest_offer.into());
            for price in (participant_lowest_offer.into()..=participant_highest_offer.into())
                .map(|idx| Price(idx))
            {
                state.array_insert(
                    6,
                    price.into(),
                    book.bid_quantity_at_price_for_participant(price, participant) as i64,
                );
            }
        }

        let vm_program_instance = vm::ProgramInstance::new(program.clone(), state);
        Self {
            vm_program_instance,
        }
    }

    pub fn execute(&mut self) {
        while let Ok(true) = self.vm_program_instance.execute_step() {}
    }

    pub fn write_result_into_book(
        &self,
        prev_book: &Book,
        result_book: &mut Book,
        participant_id: ParticipantId,
    ) {
        let mut populate_previous_orders =
            |bounds_accessor: fn(&Book, ParticipantId) -> Option<(Price, Price)>,
             side,
             quantity_accessor: fn(&Book, Price, ParticipantId) -> i64| {
                if let Some((low_price, high_price)) = bounds_accessor(prev_book, participant_id) {
                    for price in low_price.0..=high_price.0 {
                        let price = Price(price);
                        let order = Order {
                            participant: participant_id,
                            product_id: ProductId(0),
                            side,
                            quantity: quantity_accessor(prev_book, price, participant_id),
                            price,
                        };
                        result_book.insert_order(order);
                    }
                }
            };

        if self.vm_program_instance.state().array_read(9, 0) == 0 {
            populate_previous_orders(
                Book::bid_bounds_for_participant,
                Side::Bid,
                Book::bid_quantity_at_price_for_participant,
            );
        }

        if self.vm_program_instance.state().array_read(10, 0) == 0 {
            populate_previous_orders(
                Book::offer_bounds_for_participant,
                Side::Offer,
                Book::offer_quantity_at_price_for_participant,
            );
        }

        let mut add_new_orders = |array_idx, side| {
            for (idx, val) in self
                .vm_program_instance
                .state()
                .iter_touched_values(array_idx)
            {
                let order = Order {
                    participant: participant_id,
                    product_id: ProductId(0),
                    side,
                    quantity: val,
                    price: Price(idx),
                };
                result_book.update_or_insert_order(order);
            }
        };

        add_new_orders(9, Side::Bid);
        add_new_orders(10, Side::Offer);
    }
}

#[cfg(test)]
mod tests {
    use super::super::book::Level;
    use super::super::*;
    use super::*;

    #[test]
    fn construct_from_empty_book() {
        let program = vm::Program::from_instructions(&[]);
        let book = Book::default();
        let instance = ProgramInstance::new(&program, &book, ParticipantId(0));

        assert_eq!(instance.vm_program_instance.state().array_read(1, 0), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(1, 1), 0);

        assert_eq!(instance.vm_program_instance.state().array_read(2, 0), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(2, 1), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(2, 2), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(2, 3), 0);

        assert_eq!(instance.vm_program_instance.state().array_read(3, 0), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(3, 1), 0);

        assert_eq!(instance.vm_program_instance.state().array_read(4, 0), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(4, 1), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(4, 2), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(4, 3), 0);

        assert_eq!(instance.vm_program_instance.state().array_read(5, 0), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(5, 1), 0);

        assert_eq!(instance.vm_program_instance.state().array_read(6, 0), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(6, 1), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(6, 2), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(6, 3), 0);

        assert_eq!(instance.vm_program_instance.state().array_read(7, 0), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(7, 1), 0);

        assert_eq!(instance.vm_program_instance.state().array_read(8, 0), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(8, 1), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(8, 2), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(8, 3), 0);
    }

    #[test]
    fn construct_from_populated_book() {
        let program = vm::Program::from_instructions(&[]);
        let book = Book {
            levels: [
                (
                    Price(1),
                    Level {
                        orders: vec![
                            Order {
                                participant: ParticipantId(0),
                                product_id: ProductId(0),
                                price: Price(1),
                                quantity: 99,
                                side: Side::Bid,
                            },
                            Order {
                                participant: ParticipantId(1),
                                product_id: ProductId(0),
                                price: Price(1),
                                quantity: 99,
                                side: Side::Bid,
                            },
                        ],
                    },
                ),
                (
                    Price(2),
                    Level {
                        orders: vec![Order {
                            participant: ParticipantId(0),
                            product_id: ProductId(0),
                            price: Price(2),
                            quantity: 99,
                            side: Side::Bid,
                        }],
                    },
                ),
                (
                    Price(3),
                    Level {
                        orders: vec![Order {
                            participant: ParticipantId(1),
                            product_id: ProductId(0),
                            price: Price(3),
                            quantity: 99,
                            side: Side::Offer,
                        }],
                    },
                ),
            ]
            .iter()
            .cloned()
            .collect(),
        };
        let instance = ProgramInstance::new(&program, &book, ParticipantId(0));

        assert_eq!(instance.vm_program_instance.state().array_read(1, 0), 1);
        assert_eq!(instance.vm_program_instance.state().array_read(1, 1), 2);

        assert_eq!(instance.vm_program_instance.state().array_read(2, 1), 198);
        assert_eq!(instance.vm_program_instance.state().array_read(2, 2), 99);
        assert_eq!(instance.vm_program_instance.state().array_read(2, 3), 0);

        assert_eq!(instance.vm_program_instance.state().array_read(3, 0), 3);
        assert_eq!(instance.vm_program_instance.state().array_read(3, 1), 3);

        assert_eq!(instance.vm_program_instance.state().array_read(4, 1), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(4, 2), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(4, 3), 99);

        assert_eq!(instance.vm_program_instance.state().array_read(5, 0), 1);
        assert_eq!(instance.vm_program_instance.state().array_read(5, 1), 2);

        assert_eq!(instance.vm_program_instance.state().array_read(6, 1), 99);
        assert_eq!(instance.vm_program_instance.state().array_read(6, 2), 99);
        assert_eq!(instance.vm_program_instance.state().array_read(6, 3), 0);

        assert_eq!(instance.vm_program_instance.state().array_read(7, 0), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(7, 1), 0);

        assert_eq!(instance.vm_program_instance.state().array_read(8, 1), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(8, 2), 0);
        assert_eq!(instance.vm_program_instance.state().array_read(8, 3), 0);
    }

    #[test]
    fn write_result_into_book() {
        let program = vm::Program::from_instructions(&[]);
        let book = Book {
            levels: [
                (
                    Price(1),
                    Level {
                        orders: vec![
                            Order {
                                participant: ParticipantId(0),
                                product_id: ProductId(0),
                                price: Price(1),
                                quantity: 99,
                                side: Side::Bid,
                            },
                            Order {
                                participant: ParticipantId(1),
                                product_id: ProductId(0),
                                price: Price(1),
                                quantity: 99,
                                side: Side::Bid,
                            },
                        ],
                    },
                ),
                (
                    Price(2),
                    Level {
                        orders: vec![Order {
                            participant: ParticipantId(0),
                            product_id: ProductId(0),
                            price: Price(2),
                            quantity: 99,
                            side: Side::Bid,
                        }],
                    },
                ),
                (
                    Price(3),
                    Level {
                        orders: vec![Order {
                            participant: ParticipantId(0),
                            product_id: ProductId(0),
                            price: Price(3),
                            quantity: 99,
                            side: Side::Offer,
                        }],
                    },
                ),
            ]
            .iter()
            .cloned()
            .collect(),
        };
        let mut instance = ProgramInstance::new(&program, &book, ParticipantId(0));

        // Modify bid at 2 by -98 to 1, keep bid at 1
        instance
            .vm_program_instance
            .state_mut()
            .array_insert(9, 0, 0);
        instance
            .vm_program_instance
            .state_mut()
            .array_insert(9, 2, -98);
        // Expect erase offer at 3
        instance
            .vm_program_instance
            .state_mut()
            .array_insert(10, 0, 1);
        instance
            .vm_program_instance
            .state_mut()
            .array_insert(10, 4, 23);

        let mut result_book = Book::default();
        instance.write_result_into_book(&book, &mut result_book, ParticipantId(0));
        let result_book = result_book;

        assert_eq!(result_book.bid_bounds(), Some((Price(1), Price(2))));
        assert_eq!(result_book.bid_quantity_at_price(Price(1)), 99);
        assert_eq!(result_book.bid_quantity_at_price(Price(2)), 1);
        assert_eq!(
            result_book.bid_bounds_for_participant(ParticipantId(0)),
            Some((Price(1), Price(2)))
        );
        assert_eq!(
            result_book.bid_quantity_at_price_for_participant(Price(1), ParticipantId(0)),
            99
        );
        assert_eq!(
            result_book.bid_quantity_at_price_for_participant(Price(2), ParticipantId(0)),
            1
        );

        assert_eq!(result_book.offer_bounds(), Some((Price(4), Price(4))));
        assert_eq!(result_book.offer_quantity_at_price(Price(3)), 0);
        assert_eq!(result_book.offer_quantity_at_price(Price(4)), 23);
        assert_eq!(
            result_book.offer_bounds_for_participant(ParticipantId(0)),
            Some((Price(4), Price(4)))
        );
        assert_eq!(
            result_book.offer_quantity_at_price_for_participant(Price(3), ParticipantId(0)),
            0
        );
        assert_eq!(
            result_book.offer_quantity_at_price_for_participant(Price(4), ParticipantId(0)),
            23
        );
    }
}
