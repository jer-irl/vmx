use crate::protocol::{ClientDirective, ClientNotification};

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct ParticipantId(pub u64);

pub trait ParticipantPool {
    fn pop_all_directives(&mut self) -> Vec<(ParticipantId, ClientDirective)>;
    fn push_notifications_to_all(&mut self, notifications: &[(ParticipantId, ClientNotification)]);
}

pub trait Participant {
    fn handle_notification(&mut self, notification: ClientNotification);
}
