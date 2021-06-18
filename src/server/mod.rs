pub mod mocks;
pub mod tcp;

use std::sync::mpsc::Sender;

pub trait IncomingMessageHandler {
    fn sender(&self) -> Sender<IncomingMessage>;
}

pub trait Server {
    type Error: std::fmt::Debug;

    fn start_listening(&mut self) -> Result<(), Self::Error>;

    fn stop_listening(&mut self) -> Result<(), Self::Error>;

    fn request_incoming_message_notifications(&self, handler: &impl IncomingMessageHandler);

    fn send_notifications(&mut self, notifications: &[OutgoingMessage]) -> Result<(), Self::Error>;

    fn handle_pending_requests(&mut self);
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
