use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::Receiver;

use crate::{file_io::append_json_line, orders::Order, trading_engine::MatchingEngine};

pub fn process_tasks(worker_engine: Arc<Mutex<MatchingEngine>>, mut rx: Receiver<Order>) {
    tokio::spawn(async move {
        while let Some(order) = rx.recv().await {
            match append_json_line("./jsons/orders.json", &order) {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            };

            let mut new_trades = Vec::new();

            let mut eng = worker_engine.lock().unwrap();
            eng.process(order.clone(), &mut new_trades);

            for t in new_trades {
                match append_json_line("./jsons/trades.json", &t) {
                    Ok(_) => todo!(),
                    Err(_) => todo!(),
                };
            }
        }
    });
}
