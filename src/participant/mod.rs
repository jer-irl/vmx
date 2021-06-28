pub mod local;

use crate::protocol::{ClientDirective, ClientNotification};

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct ParticipantId(pub u64);

pub trait Participant {
    fn pop_directives(&self) -> Vec<ClientDirective>;
    fn push_notifications(&self, notifications: &[ClientNotification]);
}

pub trait ParticipantPool {
    fn pop_all_directives(&self) -> Vec<(ParticipantId, ClientDirective)>;
    fn push_notifications_to_all(&self, notifications: &[(ParticipantId, ClientNotification)]);
}
