use crate::{ParticipantId, Price, vm};
use super::Book;

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

        if let Some((participant_lowest_bid, participant_highest_bid)) = book.bid_bounds_for_participant(participant) {
            state.array_insert(5, 0, participant_lowest_bid.into());
            state.array_insert(5, 1, participant_highest_bid.into());
            for price in (participant_lowest_bid.into()..=participant_highest_bid.into()).map(|idx| Price(idx)) {
                state.array_insert(6, price.into(), book.bid_quantity_at_price_for_participant(price, participant) as i64);
            }
        }

        if let Some((participant_lowest_offer, participant_highest_offer)) = book.offer_bounds_for_participant(participant) {
            state.array_insert(5, 0, participant_lowest_offer.into());
            state.array_insert(5, 1, participant_highest_offer.into());
            for price in (participant_lowest_offer.into()..=participant_highest_offer.into()).map(|idx| Price(idx)) {
                state.array_insert(6, price.into(), book.bid_quantity_at_price_for_participant(price, participant) as i64);
            }
        }

        let vm_program_instance = vm::ProgramInstance::new(program.clone(), state);
        Self {
            vm_program_instance,
        }
    }

    pub fn execute(&mut self) {
        while let Ok(true) = self.vm_program_instance.execute_step() { }
    }

    pub fn write_result_into_book(&self, prev_book: &Book, result_book: &mut Book, participant_id: ParticipantId) {
        panic!("Unimplemented");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_from_empty_book() {
        panic!("Unimplemented");
    }

    #[test]
    fn construct_from_populated_book() {
        panic!("Unimplemented");
    }

    #[test]
    fn write_result_into_book() {
        panic!("Unimplemented");
    }

}
