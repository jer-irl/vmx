use std::cell::{Ref, RefCell, RefMut};

use vmx::participant::{Participant, ParticipantId, ParticipantPool};
use vmx::protocol::{ClientDirective, ClientNotification};
use vmx::vm::Program;
use vmx::ProductId;

#[derive(Default)]
pub struct MockParticipantPool {
    pub pending_directives: Vec<(ParticipantId, ClientDirective)>,
    participants: Vec<RefCell<MockParticipant>>,
}

impl MockParticipantPool {
    pub fn participant(&self, participant_id: ParticipantId) -> Option<Ref<MockParticipant>> {
        self.participants
            .iter()
            .find(|p| p.borrow().participant_id == participant_id)
            .map(|p| p.borrow())
    }

    pub fn participant_mut(
        &self,
        participant_id: ParticipantId,
    ) -> Option<RefMut<MockParticipant>> {
        self.participants
            .iter()
            .find(|p| p.borrow().participant_id == participant_id)
            .map(|p| p.borrow_mut())
    }

    pub fn add_mock_participant(&mut self, participant: MockParticipant) {
        self.participants.push(RefCell::new(participant));
    }

    fn drain_all_directives(&mut self) {
        self.pending_directives.extend(
            self.participants
                .iter_mut()
                .map(|p| {
                    let p_id = p.borrow().participant_id;
                    p.borrow_mut()
                        .pending_directives
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
            let mut participant = self
                .participants
                .iter_mut()
                .find(|p| p.borrow().participant_id == *participant_id)
                .expect("TODO")
                .borrow_mut();
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

    pub fn queue_submit_program(&mut self) {
        self.pending_directives
            .push(ClientDirective::SubmitProgram {
                product_id: self.product_id,
                program: self.program.clone(),
            });
    }

    pub fn queue_join(&mut self) {
        self.pending_directives.push(ClientDirective::Join {});
    }

    pub fn queue_leave(&mut self) {
        self.pending_directives.push(ClientDirective::Leave {});
    }

    pub fn queue_parameter_update(&mut self, parameter_idx: u64, parameter_value: i64) {
        self.pending_directives
            .push(ClientDirective::UpdateParameter {
                param_idx: parameter_idx,
                product_id: self.product_id,
                value: parameter_value,
            });
    }
}

impl Participant for MockParticipant {
    fn handle_notification(&mut self, notification: ClientNotification) {
        self.received_notifications.push(notification);
    }
}
