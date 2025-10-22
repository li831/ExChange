use trading_engine::orderbook::OrderBook;
use ordered_float::OrderedFloat;

#[test]
fn test_orderbook_creation() {
    let ob = OrderBook::new("BTCUSDT");
    assert_eq!(ob.symbol(), "BTCUSDT");
    assert_eq!(ob.bids().len(), 0);
    assert_eq!(ob.asks().len(), 0);
}

#[test]
fn test_orderbook_update_bids() {
    let mut ob = OrderBook::new("BTCUSDT");

    ob.update_bid(OrderedFloat(50000.0), OrderedFloat(1.5));
    ob.update_bid(OrderedFloat(49999.0), OrderedFloat(2.0));

    assert_eq!(ob.bids().len(), 2);

    // Best bid should be highest price
    let best_bid = ob.best_bid().unwrap();
    assert_eq!(best_bid.price, OrderedFloat(50000.0));
    assert_eq!(best_bid.quantity, OrderedFloat(1.5));
}

#[test]
fn test_orderbook_update_asks() {
    let mut ob = OrderBook::new("BTCUSDT");

    ob.update_ask(OrderedFloat(50001.0), OrderedFloat(1.0));
    ob.update_ask(OrderedFloat(50002.0), OrderedFloat(1.5));

    assert_eq!(ob.asks().len(), 2);

    // Best ask should be lowest price
    let best_ask = ob.best_ask().unwrap();
    assert_eq!(best_ask.price, OrderedFloat(50001.0));
    assert_eq!(best_ask.quantity, OrderedFloat(1.0));
}

#[test]
fn test_orderbook_spread() {
    let mut ob = OrderBook::new("BTCUSDT");

    ob.update_bid(OrderedFloat(50000.0), OrderedFloat(1.5));
    ob.update_ask(OrderedFloat(50010.0), OrderedFloat(1.0));

    let spread = ob.spread().unwrap();
    assert_eq!(spread, OrderedFloat(10.0));
}

#[test]
fn test_orderbook_midprice() {
    let mut ob = OrderBook::new("BTCUSDT");

    ob.update_bid(OrderedFloat(50000.0), OrderedFloat(1.5));
    ob.update_ask(OrderedFloat(50010.0), OrderedFloat(1.0));

    let midprice = ob.mid_price().unwrap();
    assert_eq!(midprice, OrderedFloat(50005.0));
}

#[test]
fn test_orderbook_remove_level_with_zero_quantity() {
    let mut ob = OrderBook::new("BTCUSDT");

    ob.update_bid(OrderedFloat(50000.0), OrderedFloat(1.5));
    assert_eq!(ob.bids().len(), 1);

    // Update with zero quantity should remove the level
    ob.update_bid(OrderedFloat(50000.0), OrderedFloat(0.0));
    assert_eq!(ob.bids().len(), 0);
}

#[test]
fn test_orderbook_depth() {
    let mut ob = OrderBook::new("BTCUSDT");

    // Add 5 bid levels
    for i in 0..5 {
        ob.update_bid(OrderedFloat(50000.0 - i as f64), OrderedFloat(1.0));
    }

    // Add 5 ask levels
    for i in 0..5 {
        ob.update_ask(OrderedFloat(50001.0 + i as f64), OrderedFloat(1.0));
    }

    assert_eq!(ob.bids().len(), 5);
    assert_eq!(ob.asks().len(), 5);

    // Get top 3 levels
    let top_bids = ob.get_bids(3);
    assert_eq!(top_bids.len(), 3);
    assert_eq!(top_bids[0].price, OrderedFloat(50000.0)); // Highest bid first

    let top_asks = ob.get_asks(3);
    assert_eq!(top_asks.len(), 3);
    assert_eq!(top_asks[0].price, OrderedFloat(50001.0)); // Lowest ask first
}
