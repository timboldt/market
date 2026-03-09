use strum::IntoEnumIterator;

use crate::config::{DEFAULT_PRICE, MAX_PRICE, MIN_PRICE, PRICE_EMA_ALPHA, RESOURCE_COUNT};
use crate::inventory::{self, ResourceVec};
use crate::order::{Order, Trade};
use crate::orderbook::OrderBook;
use crate::resource::Resource;

pub struct Market {
    pub books: Vec<OrderBook>,
    pub last_prices: ResourceVec,
    pub trades_this_tick: Vec<Trade>,
}

impl Market {
    pub fn new() -> Self {
        let books = Resource::iter().map(OrderBook::new).collect();
        let mut last_prices = inventory::empty_vec();
        for r in Resource::iter() {
            inventory::set(&mut last_prices, r, DEFAULT_PRICE);
        }
        Self {
            books,
            last_prices,
            trades_this_tick: Vec::new(),
        }
    }

    pub fn submit_order(&mut self, order: Order) {
        let idx = order.resource as usize;
        self.books[idx].submit(order);
    }

    pub fn clear_all(&mut self) {
        self.trades_this_tick.clear();

        for book in &mut self.books {
            let trades = book.match_orders();
            self.trades_this_tick.extend(trades);
            book.clear();
        }

        // Track which resources had trades
        let mut traded = [false; RESOURCE_COUNT];

        // Update last prices with EMA from actual trade prices
        for trade in &self.trades_this_tick {
            let idx = trade.resource as usize;
            debug_assert!(idx < RESOURCE_COUNT);
            traded[idx] = true;
            let old = self.last_prices[idx];
            self.last_prices[idx] = old * (1.0 - PRICE_EMA_ALPHA) + trade.price * PRICE_EMA_ALPHA;
        }

        // Gentle mean reversion for untouched prices — slowly pulls extreme prices
        // back toward a midpoint, preventing them from getting permanently stuck
        // at MAX_PRICE or MIN_PRICE when no trades occur.
        let midpoint = (MAX_PRICE + MIN_PRICE) / 2.0;
        let decay_rate = PRICE_EMA_ALPHA * 0.1; // Very slow: ~1.5% per tick
        for (idx, had_trade) in traded.iter().enumerate() {
            if !had_trade {
                let old = self.last_prices[idx];
                self.last_prices[idx] = old * (1.0 - decay_rate) + midpoint * decay_rate;
            }
        }

        // Clamp prices to valid range
        for p in self.last_prices.iter_mut() {
            *p = p.clamp(MIN_PRICE, MAX_PRICE);
        }
    }

    pub fn last_price(&self, r: Resource) -> f32 {
        inventory::get(&self.last_prices, r)
    }

    pub fn trade_count(&self) -> usize {
        self.trades_this_tick.len()
    }
}
