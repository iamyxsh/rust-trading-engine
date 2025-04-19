use std::{collections::HashMap, error::Error};

use orders::Order;
use trading_engine::{MatchingEngine, OrderBook, Trade};

mod file_io;
mod orders;
mod trading_engine;

fn main() -> Result<(), Box<dyn Error>> {
    let orders: Vec<Order> = file_io::read("./jsons/orders.json")?;

    let mut engine = MatchingEngine::new();
    let mut trades: Vec<Trade> = Vec::new();

    for order in orders {
        engine.process(order, &mut trades);
    }

    let mut book_snapshots: HashMap<String, OrderBook> = HashMap::new();
    for (pair, ob) in engine.books {
        book_snapshots.insert(pair.clone(), OrderBook::from(ob));
    }

    file_io::write("./jsons/orderbook.json", &book_snapshots)?;
    file_io::write("./jsons/trades.json", &trades)?;

    println!("Wrote orderbook.json and trades.json");
    Ok(())
}
