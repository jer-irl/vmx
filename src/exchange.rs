use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use chrono::{Duration, Local};
use timer::Timer;

pub use crate::auction::AuctionConfiguration;
use crate::auction::Engine;
use crate::protocol::{ClientDirective, ClientNotification};
pub use crate::server::tcp::ServerConfig;
use crate::server::{IncomingMessage, IncomingMessageHandler, Server};

pub struct Exchange<S>
where
    S: Server,
{
    engine: Engine,
    server: S,
    incoming_messages_tx: Option<Sender<IncomingMessage>>,
}

impl<S> Default for Exchange<S>
where
    S: Server,
{
    fn default() -> Self {
        panic!("Unimplemented");
    }
}

impl<S> IncomingMessageHandler for Exchange<S>
where
    S: Server,
{
    fn sender(&self) -> Sender<IncomingMessage> {
        self.incoming_messages_tx.as_ref().expect("TODO").clone()
    }
}

impl<S> Exchange<S>
where
    S: Server + Send + 'static,
{
    pub fn new(engine_config: AuctionConfiguration, server: S) -> Self {
        let engine = Engine::new(engine_config);
        Self {
            engine,
            server,
            incoming_messages_tx: None,
        }
    }

    pub fn run_forever(mut self) {
        self.server.start_listening().expect("TODO");
        let (incoming_tx, incoming_rx): (Sender<IncomingMessage>, Receiver<IncomingMessage>) =
            mpsc::channel();
        self.incoming_messages_tx = Some(incoming_tx);
        self.server.request_incoming_message_notifications(&self);
        self.server.handle_pending_requests();

        let timer = Timer::new();
        let repeat_interval =
            Duration::seconds(self.engine.config().auction_interval_seconds as i64);

        let _guard = timer.schedule(
            Local::now() + repeat_interval,
            Some(repeat_interval),
            move || {
                while let Ok(incoming_message) = incoming_rx.try_recv() {
                    let directive = ClientDirective::from(&incoming_message.bytes[..]);
                    self.engine.apply_participant_directive(&directive);
                }

                for _ in 0..self.engine.config().num_bidding_rounds {
                    self.engine.step_all_books()
                }

                let trades = self.engine.match_all_books();
                self.server
                    .send_notifications(
                        &trades
                            .iter()
                            .map(ClientNotification::from)
                            .map(ClientNotification::into)
                            .collect::<Vec<_>>()[..],
                    )
                    .expect("TODO");
            },
        );

        loop {
            thread::park();
        }
    }
}
