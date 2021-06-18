#![cfg(test)]

use std::cell::RefCell;
use std::sync::mpsc::Sender;

use super::{ClientId, IncomingMessage, IncomingMessageHandler, OutgoingMessage, Server};

#[derive(Debug, Clone, Copy)]
pub struct Error;

pub enum ServerTask {
    NewClient(ClientId),
    IncomingMessage(IncomingMessage),
}

pub struct MockServer {
    listening: bool,
    start_listening_result: Result<(), Error>,
    stop_listening_result: Result<(), Error>,
    handler_streams: RefCell<Vec<Sender<IncomingMessage>>>,
    sent_notifications: Vec<OutgoingMessage>,
    send_notifications_result: Result<(), Error>,
    pending_tasks: Vec<ServerTask>,
}

impl MockServer {
    pub fn listening(&self) -> bool {
        self.listening
    }

    pub fn set_start_listening_result(&mut self, result: Result<(), Error>) -> &mut Self {
        self.start_listening_result = result;
        self
    }

    pub fn set_stop_listening_result(&mut self, result: Result<(), Error>) -> &mut Self {
        self.stop_listening_result = result;
        self
    }

    pub fn sent_notifications(&self) -> &Vec<OutgoingMessage> {
        &self.sent_notifications
    }

    pub fn set_send_notifications_result(&mut self, result: Result<(), Error>) -> &mut Self {
        self.stop_listening_result = result;
        self
    }

    pub fn add_pending_task(&mut self, task: ServerTask) {
        self.pending_tasks.push(task);
    }
}

impl Default for MockServer {
    fn default() -> Self {
        Self {
            listening: false,
            start_listening_result: Ok(()),
            stop_listening_result: Ok(()),
            handler_streams: RefCell::default(),
            sent_notifications: Vec::default(),
            send_notifications_result: Ok(()),
            pending_tasks: Vec::default(),
        }
    }
}

impl Server for MockServer {
    type Error = Error;

    fn start_listening(&mut self) -> Result<(), Self::Error> {
        self.start_listening_result
    }

    fn stop_listening(&mut self) -> Result<(), Self::Error> {
        self.stop_listening_result
    }

    fn request_incoming_message_notifications(&self, handler: &impl IncomingMessageHandler) {
        self.handler_streams.borrow_mut().push(handler.sender());
    }

    fn send_notifications(&mut self, notifications: &[OutgoingMessage]) -> Result<(), Self::Error> {
        self.sent_notifications.extend_from_slice(notifications);
        self.send_notifications_result
    }

    fn handle_pending_requests(&mut self) {
        for task in self.pending_tasks.drain(..) {
            match task {
                ServerTask::NewClient(_client_id) => {}
                ServerTask::IncomingMessage(msg) => {
                    for sender in self.handler_streams.borrow().iter() {
                        sender.send(msg.clone()).expect("TODO");
                    }
                }
            }
        }
    }
}
