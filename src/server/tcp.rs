use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use super::{ClientId, IncomingMessage, OutgoingMessage, Server as ServerTrait};

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
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            listening_thread: None,
            client_records: vec![],
            task_channels: mpsc::channel(),
        }
    }

    fn handle_task(&mut self, task: ServerTask) -> Option<IncomingMessage> {
        match task {
            ServerTask::NewClient(mut stream) => {
                let client_id = ClientId(0); // TODO
                let record = ClientRecord {
                    client_id: client_id,
                    stream: stream.try_clone().expect("TODO"),
                };
                let send_channel = self.task_channels.0.clone();
                let join_handle = thread::spawn(move || {
                    let mut buf = [0u8; 2048];
                    while let Ok(len) = stream.read(&mut buf) {
                        send_channel
                            .send(ServerTask::IncomingMessage(IncomingMessage {
                                client_id,
                                bytes: Vec::from(&buf[0..len]),
                            }))
                            .expect("TODO");
                    }
                });
                self.client_records.push((record, join_handle));
                None
            }
            ServerTask::IncomingMessage(message) => Some(message),
        }
    }
}

impl ServerTrait for Server {
    type Error = self::Error;

    fn start_listening(&mut self) -> Result<(), Self::Error> {
        assert!(self.listening_thread.is_none());
        let listener =
            TcpListener::bind(format!("{}:{}", &self.config.ip, self.config.port)).expect("TODO");
        let listener_sending_channel = self.task_channels.0.clone();
        self.listening_thread = thread::spawn(move || {
            while let Ok((stream, _todo)) = listener.accept() {
                listener_sending_channel
                    .send(ServerTask::NewClient(stream))
                    .expect("TODO");
            }
        })
        .into();
        Ok(())
    }

    fn stop_listening(&mut self) -> Result<(), Self::Error> {
        panic!("Unimplemented");
    }

    fn drain_pending_messages(&mut self) -> Vec<IncomingMessage> {
        let mut result: Vec<IncomingMessage> = Vec::default();
        while let Ok(task) = self.task_channels.1.try_recv() {
            if let Some(message) = self.handle_task(task) {
                result.push(message);
            }
        }
        result
    }

    fn send_notifications(&mut self, notifications: &[OutgoingMessage]) -> Result<(), Self::Error> {
        for notification in notifications {
            let stream = self
                .client_records
                .iter_mut()
                .find_map(|(client_record, _join_handle)| {
                    if client_record.client_id == notification.client_id {
                        Some(&mut client_record.stream)
                    } else {
                        None
                    }
                })
                .expect("TODO");
            stream.write_all(&notification.bytes[..]).expect("TODO");
        }
        Ok(())
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

struct ClientRecord {
    client_id: ClientId,
    stream: TcpStream,
}

enum ServerTask {
    NewClient(TcpStream),
    IncomingMessage(IncomingMessage),
}
