use std::sync::mpsc::Sender;

pub mod tcp;

pub trait IncomingMessageHandler {
    fn sender(&self) -> Sender<IncomingMessage>;
}

pub trait Server {
    type Error: std::fmt::Debug;

    fn start_listening(&mut self) -> Result<(), Self::Error>;

    fn stop_listening(&mut self) -> Result<(), Self::Error>;

    fn request_incoming_message_notifications(&self, handler: &impl IncomingMessageHandler);

    fn send_notifications(&self, notifications: &[OutgoingMessage]) -> Result<(), Self::Error>;

    fn run_pending(&mut self);
}

#[derive(Clone, Copy)]
pub struct ClientId(u64);

pub struct IncomingMessage {
    pub client_id: ClientId,
    pub bytes: Vec<u8>,
}

pub struct OutgoingMessage {
    pub client_id: ClientId,
    pub bytes: Vec<u8>,
}
