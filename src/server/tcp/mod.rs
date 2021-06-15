use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

pub struct Server {
    config: ServerConfig,
    listening_thread: Option<JoinHandle<()>>,
    client_records: Vec<(ClientRecord, JoinHandle<()>)>,
    task_channels: (Sender<ServerTask>, Receiver<ServerTask>),
    incoming_message_handlers: Vec<Sender<IncomingMessage>>,
}

pub struct IncomingMessage {
    client_id: ClientId,
    bytes: Vec<u8>,
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

    pub fn start_listening(&mut self) -> Result<(), ()> {
        assert!(self.listening_thread.is_none());
        let listener =
            TcpListener::bind(format!("{}{}", &self.config.ip, self.config.port)).unwrap();
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

    pub fn stop_listening(&mut self) -> Result<(), ()> {
        panic!("Unimplemented");
    }

    pub fn request_incoming_message_notifications(
        &mut self,
        sending_channel: Sender<IncomingMessage>,
    ) {
        self.incoming_message_handlers.push(sending_channel);
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
    ip: String,
    port: u16,
}

#[derive(Clone, Copy)]
struct ClientId(u64);

struct ClientRecord {
    client_id: ClientId,
}

enum ServerTask {
    NewClient(TcpStream),
    IncomingMessage { client_id: ClientId, bytes: Vec<u8> },
}
