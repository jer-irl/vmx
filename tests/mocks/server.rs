use vmx::participant::{ParticipantId, ParticipantPool};
use vmx::protocol::{ClientDirective, ClientNotification};
use vmx::server::{ClientId, IncomingMessage, OutgoingMessage, Server};

#[derive(Debug, Clone, Copy)]
pub struct Error;
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mock error")
    }
}
impl std::error::Error for Error {}

pub enum ServerTask {
    NewClient(ClientId),
    IncomingMessage(IncomingMessage),
}

pub struct MockServer {
    listening: bool,
    start_listening_result: Result<(), Error>,
    stop_listening_result: Result<(), Error>,
    sent_notifications: Vec<OutgoingMessage>,
    send_notifications_result: Result<(), Error>,
    pending_tasks: Vec<ServerTask>,
}

impl ParticipantPool for MockServer {
    fn push_notifications_to_all(&mut self, notifications: &[(ParticipantId, ClientNotification)]) {
        todo!();
    }

    fn pop_all_directives(&mut self) -> Vec<(ParticipantId, ClientDirective)> {
        todo!();
    }
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

    fn send_notifications(&mut self, notifications: &[OutgoingMessage]) -> Result<(), Self::Error> {
        self.sent_notifications.extend_from_slice(notifications);
        self.send_notifications_result
    }

    fn drain_pending_messages(&mut self) -> Vec<IncomingMessage> {
        let mut result: Vec<IncomingMessage> = Vec::default();
        for task in self.pending_tasks.drain(..) {
            match task {
                ServerTask::NewClient(_client_id) => {}
                ServerTask::IncomingMessage(msg) => {
                    result.push(msg);
                }
            }
        }
        result
    }
}
