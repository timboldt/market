use crate::order::{Order, Side, Trade};
use crate::resource::Resource;

pub struct OrderBook {
    pub resource: Resource,
    pub bids: Vec<Order>, // sorted highest price first
    pub asks: Vec<Order>, // sorted lowest price first
}

impl OrderBook {
    pub fn new(resource: Resource) -> Self {
        Self {
            resource,
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    pub fn submit(&mut self, order: Order) {
        match order.side {
            Side::Buy => {
                let pos = self
                    .bids
                    .binary_search_by(|o| {
                        o.price
                            .partial_cmp(&order.price)
                            .unwrap()
                            .reverse()
                            .then(o.id.cmp(&order.id))
                    })
                    .unwrap_or_else(|e| e);
                self.bids.insert(pos, order);
            }
            Side::Sell => {
                let pos = self
                    .asks
                    .binary_search_by(|o| {
                        order
                            .price
                            .partial_cmp(&o.price)
                            .unwrap()
                            .reverse()
                            .then(o.id.cmp(&order.id))
                    })
                    .unwrap_or_else(|e| e);
                self.asks.insert(pos, order);
            }
        }
    }

    pub fn match_orders(&mut self) -> Vec<Trade> {
        let mut trades = Vec::new();

        while !self.bids.is_empty() && !self.asks.is_empty() {
            let best_bid = self.bids[0].price;
            let best_ask = self.asks[0].price;

            if best_bid < best_ask {
                break;
            }

            // Trade at midpoint (both sides influence price)
            let trade_price = (best_bid + best_ask) / 2.0;
            let trade_qty = self.bids[0].quantity.min(self.asks[0].quantity);

            trades.push(Trade {
                resource: self.resource,
                price: trade_price,
                quantity: trade_qty,
                buyer_id: self.bids[0].agent_id,
                seller_id: self.asks[0].agent_id,
            });

            self.bids[0].quantity -= trade_qty;
            self.asks[0].quantity -= trade_qty;

            if self.bids[0].quantity < 0.001 {
                self.bids.remove(0);
            }
            if self.asks[0].quantity < 0.001 {
                self.asks.remove(0);
            }
        }

        trades
    }

    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
    }
}
