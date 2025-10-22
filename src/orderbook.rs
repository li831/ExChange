use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

/// Represents a single price level in the order book
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PriceLevel {
    pub price: OrderedFloat<f64>,
    pub quantity: OrderedFloat<f64>,
}

impl PriceLevel {
    pub fn new(price: f64, quantity: f64) -> Self {
        Self {
            price: OrderedFloat(price),
            quantity: OrderedFloat(quantity),
        }
    }
}

/// Order book data structure for managing bids and asks
/// Bids are sorted in descending order (highest price first)
/// Asks are sorted in ascending order (lowest price first)
#[derive(Debug, Clone)]
pub struct OrderBook {
    symbol: String,
    /// Bids: price -> quantity (sorted descending by price)
    bids: BTreeMap<OrderedFloat<f64>, OrderedFloat<f64>>,
    /// Asks: price -> quantity (sorted ascending by price)
    asks: BTreeMap<OrderedFloat<f64>, OrderedFloat<f64>>,
    last_update_id: Option<u64>,
}

impl OrderBook {
    /// Create a new empty order book for a symbol
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            last_update_id: None,
        }
    }

    /// Get the symbol for this order book
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Get all bids (returns reference to BTreeMap)
    pub fn bids(&self) -> &BTreeMap<OrderedFloat<f64>, OrderedFloat<f64>> {
        &self.bids
    }

    /// Get all asks (returns reference to BTreeMap)
    pub fn asks(&self) -> &BTreeMap<OrderedFloat<f64>, OrderedFloat<f64>> {
        &self.asks
    }

    /// Update or insert a bid level
    /// If quantity is 0, the level is removed
    pub fn update_bid(&mut self, price: OrderedFloat<f64>, quantity: OrderedFloat<f64>) {
        if quantity == OrderedFloat(0.0) {
            self.bids.remove(&price);
        } else {
            self.bids.insert(price, quantity);
        }
    }

    /// Update or insert an ask level
    /// If quantity is 0, the level is removed
    pub fn update_ask(&mut self, price: OrderedFloat<f64>, quantity: OrderedFloat<f64>) {
        if quantity == OrderedFloat(0.0) {
            self.asks.remove(&price);
        } else {
            self.asks.insert(price, quantity);
        }
    }

    /// Get the best bid (highest bid price)
    pub fn best_bid(&self) -> Option<PriceLevel> {
        self.bids.iter().next_back().map(|(price, quantity)| PriceLevel {
            price: *price,
            quantity: *quantity,
        })
    }

    /// Get the best ask (lowest ask price)
    pub fn best_ask(&self) -> Option<PriceLevel> {
        self.asks.iter().next().map(|(price, quantity)| PriceLevel {
            price: *price,
            quantity: *quantity,
        })
    }

    /// Calculate the spread (best ask - best bid)
    pub fn spread(&self) -> Option<OrderedFloat<f64>> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some(ask.price - bid.price),
            _ => None,
        }
    }

    /// Calculate the mid price ((best ask + best bid) / 2)
    pub fn mid_price(&self) -> Option<OrderedFloat<f64>> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some((ask.price + bid.price) / 2.0),
            _ => None,
        }
    }

    /// Get top N bid levels (sorted by price descending)
    pub fn get_bids(&self, depth: usize) -> Vec<PriceLevel> {
        self.bids
            .iter()
            .rev() // Reverse to get highest prices first
            .take(depth)
            .map(|(price, quantity)| PriceLevel {
                price: *price,
                quantity: *quantity,
            })
            .collect()
    }

    /// Get top N ask levels (sorted by price ascending)
    pub fn get_asks(&self, depth: usize) -> Vec<PriceLevel> {
        self.asks
            .iter()
            .take(depth)
            .map(|(price, quantity)| PriceLevel {
                price: *price,
                quantity: *quantity,
            })
            .collect()
    }

    /// Update the last update ID (for tracking Binance updates)
    pub fn set_last_update_id(&mut self, update_id: u64) {
        self.last_update_id = Some(update_id);
    }

    /// Get the last update ID
    pub fn last_update_id(&self) -> Option<u64> {
        self.last_update_id
    }

    /// Clear all bids and asks
    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.last_update_id = None;
    }

    /// Get the total bid quantity across all levels
    pub fn total_bid_quantity(&self) -> OrderedFloat<f64> {
        self.bids.values().copied().sum()
    }

    /// Get the total ask quantity across all levels
    pub fn total_ask_quantity(&self) -> OrderedFloat<f64> {
        self.asks.values().copied().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_level_creation() {
        let level = PriceLevel::new(100.0, 1.5);
        assert_eq!(level.price, OrderedFloat(100.0));
        assert_eq!(level.quantity, OrderedFloat(1.5));
    }

    #[test]
    fn test_orderbook_total_quantities() {
        let mut ob = OrderBook::new("BTCUSDT");
        ob.update_bid(OrderedFloat(100.0), OrderedFloat(1.0));
        ob.update_bid(OrderedFloat(99.0), OrderedFloat(2.0));
        ob.update_ask(OrderedFloat(101.0), OrderedFloat(1.5));
        ob.update_ask(OrderedFloat(102.0), OrderedFloat(2.5));

        assert_eq!(ob.total_bid_quantity(), OrderedFloat(3.0));
        assert_eq!(ob.total_ask_quantity(), OrderedFloat(4.0));
    }
}
