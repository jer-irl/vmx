mod common_programs;
mod mocks;

use mocks::participant::MockParticipant;
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
    let _participant = MockParticipant::new(ParticipantId(0), ProductId(0), program);
    todo!();
}

#[test]
fn parameters_updated() {
    todo!();
}

#[test]
fn prevent_self_crossing() {
    todo!();
}
