pub mod mocks;
pub mod tcp;

pub trait Server {
    type Error: std::fmt::Debug;

    fn start_listening(&mut self) -> Result<(), Self::Error>;

    fn stop_listening(&mut self) -> Result<(), Self::Error>;

    fn drain_pending_messages(&mut self) -> Vec<IncomingMessage>;

    fn send_notifications(&mut self, notifications: &[OutgoingMessage]) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ClientId(u64);

#[derive(Clone)]
pub struct IncomingMessage {
    pub client_id: ClientId,
    pub bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct OutgoingMessage {
    pub client_id: ClientId,
    pub bytes: Vec<u8>,
}
