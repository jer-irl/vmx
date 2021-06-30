mod common_programs;
mod mocks;

use mocks::participant::{MockParticipant, MockParticipantPool};
use vmx::exchange::{AuctionConfiguration, Exchange};
use vmx::vm::Program;
use vmx::Price;
use vmx::{participant::ParticipantId, ProductId};

#[test]
fn program_quotes_applied() {
    let program = Program::from_instructions(&common_programs::replace_quotes(
        Price(100),
        100,
        Price(200),
        100,
    ));
    let participant = MockParticipant::new(ParticipantId(0), ProductId(0), program);
    let mut participant_pool = MockParticipantPool::default();
    participant_pool.add_mock_participant(participant);

    let mut exchange = Exchange::new(AuctionConfiguration::default(), participant_pool);
    exchange.step().expect("TODO");

    assert_eq!(
        exchange
            .participant_pool()
            .participant(ParticipantId(0))
            .unwrap()
            .received_notifications
            .len(),
        0
    );
}

#[test]
fn parameters_updated() {
    todo!();
}

#[test]
fn prevent_self_crossing() {
    todo!();
}
