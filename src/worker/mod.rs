use std::{
    error,
    sync::{Arc, Mutex},
};

use log::error;
use tokio::sync::mpsc::Receiver;

use crate::{
    file_io::{self, append_json_to_array},
    orders::Order,
    trading_engine::{MatchingEngine, Trade},
};

pub fn process_tasks(worker_engine: Arc<Mutex<MatchingEngine>>, mut rx: Receiver<Order>) {
    tokio::spawn(async move {
        while let Some(order) = rx.recv().await {
            if let Err(e) = append_json_to_array("./jsons/orders.json", &order) {
                error!("Failed to append to orders.json: {}", e);
            }

            let mut new_trades: Vec<Trade> = Vec::new();
            {
                let mut eng = worker_engine.lock().unwrap();
                eng.process(order.clone(), &mut new_trades);

                if let Err(e) = file_io::write("./jsons/orderbook.json", &eng.books) {
                    error!("Failed to write orderbook.json: {}", e);
                }
            }

            for trade in new_trades {
                if let Err(e) = append_json_to_array("./jsons/trades.json", &trade) {
                    error!("Failed to append to trades.json: {}", e);
                }
            }
        }
    });
}
