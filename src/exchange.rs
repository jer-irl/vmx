pub use crate::auction::AuctionConfiguration;
use crate::auction::{Engine, Side};
use crate::participant::ParticipantId;
use crate::participant::ParticipantPool;
use crate::protocol::ClientNotification;

pub struct Exchange<P>
where
    P: ParticipantPool,
{
    engine: Engine,
    participant_pool: P,
}

impl<P> Default for Exchange<P>
where
    P: ParticipantPool,
{
    fn default() -> Self {
        todo!();
    }
}

impl<P> Exchange<P>
where
    P: ParticipantPool,
{
    pub fn new(engine_config: AuctionConfiguration, participant_pool: P) -> Self {
        let engine = Engine::new(engine_config);
        Self {
            engine,
            participant_pool,
        }
    }

    pub fn step(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let pending_client_messages = self.participant_pool.pop_all_directives();
        for (participant_id, directive) in pending_client_messages {
            self.engine
                .apply_participant_directive(participant_id, &directive);
        }

        self.engine.step_all_books();
        let trades = self.engine.match_all_books();

        let mut notifications: Vec<(ParticipantId, ClientNotification)> = Vec::default();
        for trade in trades {
            let seller_notification = ClientNotification::Trade {
                product_id: trade.product_id,
                quantity: trade.quantity,
                side: Side::Offer,
            };
            notifications.push((trade.seller, seller_notification));
            let buyer_notification = ClientNotification::Trade {
                product_id: trade.product_id,
                quantity: trade.quantity,
                side: Side::Bid,
            };
            notifications.push((trade.buyer, buyer_notification));
        }

        self.participant_pool
            .push_notifications_to_all(&notifications[..]);

        Ok(())
    }
}
