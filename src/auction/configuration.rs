pub struct AuctionConfiguration {
    pub num_bidding_rounds: u64,
    pub auction_interval_seconds: u64,
}

impl Default for AuctionConfiguration {
    fn default() -> Self {
        Self {
            num_bidding_rounds: 5,
            auction_interval_seconds: 1,
        }
    }
}
