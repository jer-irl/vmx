use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

#[derive(Debug)]
pub enum Error {
    Net,
    Thread,
}

pub struct Server {
    config: ServerConfig,
    listening_thread: Option<JoinHandle<()>>,
    client_records: Vec<(ClientRecord, JoinHandle<()>)>,
    task_channels: (Sender<ServerTask>, Receiver<ServerTask>),
    incoming_message_handlers: Vec<Sender<IncomingMessage>>,
}

pub struct IncomingMessage {
    pub client_id: ClientId,
    pub bytes: Vec<u8>,
}

pub struct OutgoingMessage {
    pub client_id: ClientId,
    pub bytes: Vec<u8>,
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            listening_thread: None,
            client_records: vec![],
            task_channels: mpsc::channel(),
            incoming_message_handlers: vec![],
        }
    }

    pub fn start_listening(&mut self) -> Result<(), Error> {
        assert!(self.listening_thread.is_none());
        let listener =
            TcpListener::bind(format!("{}:{}", &self.config.ip, self.config.port)).expect("TODO");
        let listener_sending_channel = self.task_channels.0.clone();
        self.listening_thread = thread::spawn(move || {
            while let Ok((stream, _todo)) = listener.accept() {
                listener_sending_channel
                    .send(ServerTask::NewClient(stream))
                    .unwrap();
            }
        })
        .into();
        Ok(())
    }

    pub fn stop_listening(&mut self) -> Result<(), Error> {
        panic!("Unimplemented");
    }

    pub fn request_incoming_message_notifications(
        &mut self,
        sending_channel: Sender<IncomingMessage>,
    ) {
        self.incoming_message_handlers.push(sending_channel);
    }

    pub fn send_notifications(&self, notifications: &[OutgoingMessage]) -> Result<(), Error> {
        panic!("Unimplemented");
    }

    pub fn run(&mut self) {
        while let Ok(task) = self.task_channels.1.recv() {
            self.handle_task(task)
        }
    }

    fn handle_task(&mut self, task: ServerTask) {
        match task {
            ServerTask::NewClient(mut stream) => {
                let client_id = ClientId(0); // TODO
                let record = ClientRecord { client_id };
                let send_channel = self.task_channels.0.clone();
                let join_handle = thread::spawn(move || {
                    let mut buf = [0u8; 2048];
                    while let Ok(len) = stream.read(&mut buf) {
                        send_channel
                            .send(ServerTask::IncomingMessage {
                                client_id,
                                bytes: Vec::from(&buf[0..len]),
                            })
                            .expect("TODO");
                    }
                });
                self.client_records.push((record, join_handle));
            }
            ServerTask::IncomingMessage { client_id, bytes } => {
                for handler in &self.incoming_message_handlers {
                    handler
                        .send(IncomingMessage {
                            client_id,
                            bytes: bytes.clone(),
                        })
                        .expect("TODO");
                }
            }
        }
    }
}

pub struct ServerConfig {
    pub ip: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_owned(),
            port: 8080,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ClientId(u64);

struct ClientRecord {
    client_id: ClientId,
}

enum ServerTask {
    NewClient(TcpStream),
    IncomingMessage { client_id: ClientId, bytes: Vec<u8> },
}
