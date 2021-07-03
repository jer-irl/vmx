mod helpers;
mod mocks;

use helpers::program_builders::ProgramBuilder;
use mocks::participant::{MockParticipant, MockParticipantPool};
use vmx::auction::Side;
use vmx::exchange::{AuctionConfiguration, Exchange};
use vmx::participant::ParticipantId;
use vmx::protocol::ClientNotification;
use vmx::{Price, ProductId};

#[test]
fn simple_matching() {
    let program1 = ProgramBuilder::new().replace_asks(Price(100), 100).build();
    let program2 = ProgramBuilder::new().replace_bids(Price(100), 100).build();
    let participant1_id = ParticipantId(1);
    let participant2_id = ParticipantId(2);
    let product_id = ProductId(0);
    let participant1 = MockParticipant::new(participant1_id, product_id, program1);
    let participant2 = MockParticipant::new(participant2_id, product_id, program2);
    let mut participant_pool = MockParticipantPool::default();
    participant_pool.add_mock_participant(participant1);
    participant_pool.add_mock_participant(participant2);

    let mut exchange = Exchange::new(AuctionConfiguration::default(), participant_pool);

    exchange
        .participant_pool()
        .participant_mut(participant1_id)
        .unwrap()
        .queue_join();
    exchange
        .participant_pool()
        .participant_mut(participant2_id)
        .unwrap()
        .queue_join();

    exchange.step().expect("TODO");

    assert_eq!(
        exchange
            .participant_pool()
            .participant(participant1_id)
            .unwrap()
            .received_notifications
            .len(),
        0
    );
    assert_eq!(
        exchange
            .participant_pool()
            .participant(participant2_id)
            .unwrap()
            .received_notifications
            .len(),
        0
    );

    exchange
        .participant_pool()
        .participant_mut(participant1_id)
        .unwrap()
        .queue_submit_program();

    exchange
        .participant_pool()
        .participant_mut(participant2_id)
        .unwrap()
        .queue_submit_program();

    exchange.apply_participant_directives();
    exchange.step_all_books_one_auction();

    let trades = exchange.match_all_books();
    assert_eq!(trades.len(), 2);

    exchange.send_trade_notifications(trades);

    {
        let participant1_notifications = &exchange
            .participant_pool()
            .participant(participant1_id)
            .unwrap()
            .received_notifications;
        assert_eq!(participant1_notifications.len(), 1);
        assert_eq!(
            *participant1_notifications.get(0).unwrap(),
            ClientNotification::Trade {
                product_id,
                price: Price(100),
                quantity: 100,
                side: Side::Offer,
            }
        );
    }

    {
        let participant2_notifications = &exchange
            .participant_pool()
            .participant(participant2_id)
            .unwrap()
            .received_notifications;
        assert_eq!(participant2_notifications.len(), 1);
        assert_eq!(
            *participant2_notifications.get(0).unwrap(),
            ClientNotification::Trade {
                product_id,
                price: Price(100),
                quantity: 100,
                side: Side::Bid,
            }
        );
    }

    exchange
        .participant_pool()
        .participant_mut(participant1_id)
        .unwrap()
        .queue_leave();
    exchange
        .participant_pool()
        .participant_mut(participant2_id)
        .unwrap()
        .queue_leave();
    exchange.apply_participant_directives();
    exchange.step_all_books_one_auction();
    let trades = exchange.match_all_books();
    assert!(trades.is_empty());
}

#[test]
fn parameters_updated() {
    let seller_starting_price = Price(200);
    let seller_ending_price = Price(100);
    let buyer_price = Price(100);
    let seller_price_parameter = 1;

    let buyer_program = ProgramBuilder::new().replace_bids(buyer_price, 100).build();
    let seller_program = ProgramBuilder::new()
        .replace_quotes_with_parameter_price(Side::Offer, seller_price_parameter, 100)
        .build();

    let buyer_id = ParticipantId(1);
    let seller_id = ParticipantId(2);
    let product_id = ProductId(1);

    let mut buyer = MockParticipant::new(buyer_id, product_id, buyer_program);
    buyer.queue_join();
    buyer.queue_submit_program();
    let mut seller = MockParticipant::new(seller_id, product_id, seller_program);
    seller.queue_join();
    seller.queue_submit_program();
    seller.queue_parameter_update(seller_price_parameter, seller_starting_price.0 as i64);
    let mut participant_pool = MockParticipantPool::default();
    participant_pool.add_mock_participant(buyer);
    participant_pool.add_mock_participant(seller);

    let mut exchange = Exchange::new(AuctionConfiguration::default(), participant_pool);

    exchange.apply_participant_directives();
    exchange.step_all_books_one_auction();
    let trades = exchange.match_all_books();
    assert!(trades.is_empty());

    exchange
        .participant_pool()
        .participant_mut(seller_id)
        .unwrap()
        .queue_parameter_update(seller_price_parameter, seller_ending_price.0 as i64);

    exchange.apply_participant_directives();
    exchange.step_all_books_one_auction();
    let trades = exchange.match_all_books();
    assert_eq!(trades.len(), 2);
    assert_eq!(trades.get(0).unwrap().price, seller_ending_price);
}

#[test]
fn prevent_self_crossing() {
    todo!();
}
