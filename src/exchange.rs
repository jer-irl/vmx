use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use chrono::{Duration, Local};
use timer::Timer;

pub use crate::auction::AuctionConfiguration;
use crate::auction::Engine;
use crate::protocol::{ClientDirective, ClientNotification};
pub use crate::server::tcp::ServerConfig;
use crate::server::{IncomingMessage, Server};

pub struct Exchange<S>
where
    S: Server,
{
    engine: Engine,
    server: S,
}

impl<S> Default for Exchange<S>
where
    S: Server,
{
    fn default() -> Self {
        panic!("Unimplemented");
    }
}

impl<S> Exchange<S>
where
    S: Server + Send + 'static,
{
    pub fn new(engine_config: AuctionConfiguration, server: S) -> Self {
        let engine = Engine::new(engine_config);
        Self { engine, server }
    }

    pub fn step(&mut self) {
        panic!("Unimplemented");
    }
}
