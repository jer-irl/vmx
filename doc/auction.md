# Auction Overview

The matching engine will be a simple multi-round periodic double auction with a VCG trade price mechanism.
Each bidding round of a single auction, without matching bids and offers, the engine will publish the state of the book to all participants.
Participants then have the opportunity to revise their previously published orders, at a cost paid to the exchange.
This cost will be a function of the distance and direction of the price change.
Price revisions are calculated by a program previously submitted by each market participant.
Parameters to a participant's program may be revised before the first bidding round.
Costs may be associated with parameter updates and/or program submission and/or program execution time.
Program execution cost is measured through a "gas" mechanism similar to the Ethereum Virtual Machine.

If there is a tie on a level, the orders are filled by order size priority (pro-rata).

## Matching engine parameters

### Auction periodicity

### Bidding rounds per auction

### Order revision cost function

### Program submission cost

### Program paremeter cost

### Parameter update cost

### Program execution (gas) cost

## Exchange considerations

Costs levied against participants must outweigh the exchange-paid costs of the VCG mechanism.

## Participant benefits

### Latency advantage mitigation

As a discrete periodic auction, there is no possible advantage associated with minimizing latency.
This "levels the playing field," so sophisticated entities with capital to invest in trading infrastructure cannot win "by default" against smaller investors.

### Responsiveness to market state at predictable cost

The costs of revising prices are easily calculatable.
