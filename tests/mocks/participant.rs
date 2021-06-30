use vmx::participant::{Participant, ParticipantId, ParticipantPool};
use vmx::protocol::{ClientDirective, ClientNotification};
use vmx::vm::Program;
use vmx::ProductId;

#[derive(Default)]
pub struct MockParticipantPool {
    pub pending_directives: Vec<(ParticipantId, ClientDirective)>,
    participants: Vec<MockParticipant>,
}

impl MockParticipantPool {
    pub fn participant(&self, participant_id: ParticipantId) -> Option<&MockParticipant> {
        self.participants
            .iter()
            .find(|p| p.participant_id == participant_id)
    }

    pub fn add_mock_participant(&mut self, participant: MockParticipant) {
        let participant_id = participant.participant_id;
        let product_id = participant.product_id;
        let program = participant.program.clone();
        self.participants.push(participant);
        self.pending_directives
            .push((participant_id, ClientDirective::Join {}));
        self.pending_directives.push((
            participant_id,
            ClientDirective::SubmitProgram {
                product_id,
                program,
            },
        ))
    }

    fn drain_all_directives(&mut self) {
        self.pending_directives.extend(
            self.participants
                .iter_mut()
                .map(|p| {
                    let p_id = p.participant_id;
                    p.pending_directives
                        .drain(..)
                        .map(|directive| (p_id, directive))
                        .collect::<Vec<_>>()
                })
                .flatten(),
        );
    }
}

impl ParticipantPool for MockParticipantPool {
    fn pop_all_directives(&mut self) -> Vec<(ParticipantId, ClientDirective)> {
        self.drain_all_directives();
        self.pending_directives.drain(..).collect()
    }

    fn push_notifications_to_all(&mut self, notifications: &[(ParticipantId, ClientNotification)]) {
        for (participant_id, notification) in notifications {
            let participant = self
                .participants
                .iter_mut()
                .find(|p| p.participant_id == *participant_id)
                .expect("TODO");
            participant.handle_notification(notification.clone());
        }
    }
}

pub struct MockParticipant {
    participant_id: ParticipantId,
    product_id: ProductId,
    program: Program,
    pending_directives: Vec<ClientDirective>,
    pub received_notifications: Vec<ClientNotification>,
}

impl MockParticipant {
    pub fn new(participant_id: ParticipantId, product_id: ProductId, program: Program) -> Self {
        Self {
            participant_id,
            product_id,
            program,
            pending_directives: Vec::default(),
            received_notifications: Vec::default(),
        }
    }
}

impl Participant for MockParticipant {
    fn handle_notification(&mut self, notification: ClientNotification) {
        self.received_notifications.push(notification);
    }
}
