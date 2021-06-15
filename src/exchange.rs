use std::sync::mpsc;
use std::thread;

use chrono::{Duration, Local};
use timer::Timer;

use crate::auction::{configuration::AuctionConfiguration, Engine};
use crate::protocol::{ClientDirective, ClientNotification};
use crate::server::tcp::{Server, ServerConfig};

pub struct Exchange {
	engine: Engine,
	server: Server,
}

impl Default for Exchange {
	fn default() -> Self {
		panic!("Unimplemented");
	}
}

impl Exchange {
	pub fn new(engine_config: AuctionConfiguration, server_config: ServerConfig) -> Self {
		panic!("Unimplemented");
	}

	pub fn run_forever(mut self) {
		self.server.start_listening().expect("TODO");
		let (incoming_tx, incoming_rx) = mpsc::channel();
		self.server
			.request_incoming_message_notifications(incoming_tx);
		self.server.run();

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
