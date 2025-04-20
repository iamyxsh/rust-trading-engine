use actix_web::{middleware::Logger, web, App, HttpServer};

use routes::{orders_service, trades_service};
use std::{
    collections::HashMap,
    error::Error,
    sync::{atomic::AtomicUsize, Arc, Mutex},
};
use tokio::sync::{broadcast, mpsc};
use trading_engine::{
    matching_engine::{MatchingEngine, Trade},
    orderbook::OrderBook,
};
use worker::process_tasks;

use orders::{Order, TypeOp};

mod file_io;
mod orders;
mod routes;
mod trading_engine;
mod worker;
mod ws;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let orders: Vec<Order> = file_io::read("./jsons/orders.json")?;
    let orders_len = orders
        .iter()
        .filter(|o| o.type_op == TypeOp::Create)
        .count()
        + 1;

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

    let (ws_tx, _) = broadcast::channel::<String>(256);

    let ws_tx_arc = Arc::new(ws_tx);

    let engine = Arc::new(Mutex::new(engine.clone()));
    let (tx, rx) = mpsc::channel::<Order>(256);

    let id_counter = Arc::new(AtomicUsize::new(orders_len));

    let worker_engine = engine.clone();

    process_tasks(worker_engine, rx, ws_tx_arc.clone());

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(tx.clone()))
            .app_data(web::Data::new(id_counter.clone()))
            .app_data(web::Data::new(engine.clone()))
            .app_data(web::Data::new(ws_tx_arc.clone()))
            .service(orders_service::order_scope())
            .service(trades_service::trades_scope())
            .service(ws::ws_index)
    })
    .bind(("0.0.0.0", 5050))?
    .run()
    .await?;

    Ok(())
}
