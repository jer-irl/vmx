pub use crate::auction::AuctionConfiguration;
use crate::auction::{Engine, Side};
use crate::protocol::json::JsonProtocol;
use crate::protocol::{ClientNotification, WireProtocol};
pub use crate::server::tcp::ServerConfig;
use crate::server::{ClientId, OutgoingMessage, Server};

pub trait Exchange {
    type WireProtocol: WireProtocol;
}

pub struct JsonExchange<S>
where
    S: Server,
{
    engine: Engine,
    server: S,
}

impl<S> Default for JsonExchange<S>
where
    S: Server,
{
    fn default() -> Self {
        panic!("Unimplemented");
    }
}

impl<S> Exchange for JsonExchange<S>
where
    S: Server,
{
    type WireProtocol = JsonProtocol;
}

impl<S> JsonExchange<S>
where
    S: Server,
{
    pub fn new(engine_config: AuctionConfiguration, server: S) -> Self {
        let engine = Engine::new(engine_config);
        Self { engine, server }
    }

    pub fn step(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let pending_client_messages = self.server.drain_pending_messages();
        for message in pending_client_messages {
            let directive = <Self as Exchange>::WireProtocol::try_client_directive_from_bytes(
                &message.bytes[..],
            )?;
            self.engine.apply_participant_directive(&directive);
        }

        self.engine.step_all_books();
        let trades = self.engine.match_all_books();

        let mut notifications: Vec<ClientNotification> = Vec::default();
        for trade in trades {
            let seller_notification = ClientNotification::Trade {
                product_id: trade.product_id,
                quantity: trade.quantity,
                side: Side::Offer,
            };
            notifications.push(seller_notification);
            let buyer_notification = ClientNotification::Trade {
                product_id: trade.product_id,
                quantity: trade.quantity,
                side: Side::Bid,
            };
            notifications.push(buyer_notification);
        }

        self.server
            .send_notifications(
                &notifications
                    .into_iter()
                    .filter_map(|n| {
                        <Self as Exchange>::WireProtocol::try_client_notification_to_bytes(&n).ok()
                    })
                    .map(|bytes| OutgoingMessage {
                        client_id: ClientId(0), // TODO
                        bytes,
                    })
                    .collect::<Vec<_>>()[..],
            )
            .expect("TODO");

        Ok(())
    }
}
