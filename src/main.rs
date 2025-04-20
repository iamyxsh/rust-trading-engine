use actix_web::{web, App, HttpServer};

use routes::orders_service;
use std::{
    collections::HashMap,
    env,
    error::Error,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;
use worker::process_tasks;

use orders::Order;
use trading_engine::{MatchingEngine, OrderBook, Trade};

mod file_io;
mod orders;
mod routes;
mod trading_engine;
mod worker;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let orders: Vec<Order> = file_io::read("./jsons/orders.json")?;

    let mut engine = MatchingEngine::new();
    let mut trades: Vec<Trade> = Vec::new();

    for order in orders {
        engine.process(order, &mut trades);
    }

    let mut book_snapshots: HashMap<String, OrderBook> = HashMap::new();
    for (pair, ob) in engine.books.iter() {
        book_snapshots.insert(pair.clone(), OrderBook::from(ob.clone()));
    }

    file_io::write("./jsons/orderbook.json", &book_snapshots)?;
    file_io::write("./jsons/trades.json", &trades)?;

    println!("Wrote orderbook.json and trades.json");

    let engine = Arc::new(Mutex::new(engine.clone()));
    let (tx, mut rx) = mpsc::channel::<Order>(256);

    let worker_engine = engine.clone();

    process_tasks(worker_engine, rx);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tx.clone()))
            .app_data(web::Data::new(tx.clone()))
            .app_data(web::Data::new(engine.clone()))
            .service(orders_service::order_scope())
    })
    .bind(("0.0.0.0", 5050))?
    .run()
    .await?;

    Ok(())
}
