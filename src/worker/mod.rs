use std::sync::{Arc, Mutex};

use log::error;
use serde_json::json;
use tokio::sync::{broadcast, mpsc::Receiver};

use crate::{
    file_io::{self, append_json_to_array},
    orders::Order,
    trading_engine::matching_engine::{MatchingEngine, Trade},
};

pub fn process_tasks(
    worker_engine: Arc<Mutex<MatchingEngine>>,
    mut rx: Receiver<Order>,
    ws_tx: Arc<tokio::sync::broadcast::Sender<String>>,
) {
    tokio::spawn(async move {
        while let Some(order) = rx.recv().await {
            if let Err(e) = append_json_to_array("./jsons/orders.json", &order) {
                error!("Failed to append to orders.json: {}", e);
            }

            let evt_type = match order.type_op {
                crate::orders::TypeOp::Create => "order_create",
                crate::orders::TypeOp::Delete => "order_delete",
            };
            let order_msg = json!({ "type": evt_type, "order": order });
            let _ = ws_tx.send(order_msg.to_string());

            let mut new_trades: Vec<Trade> = Vec::new();
            {
                let mut eng = worker_engine.lock().unwrap();
                eng.process(order.clone(), &mut new_trades);

                if let Err(e) = file_io::write("./jsons/orderbook.json", &eng.books) {
                    error!("Failed to write orderbook.json: {}", e);
                }

                let snapshot = json!({
                    "type": "orderbook_snapshot",
                    "books": eng.books,
                });
                let _ = ws_tx.send(snapshot.to_string());
            }

            for trade in new_trades {
                if let Err(e) = append_json_to_array("./jsons/trades.json", &trade) {
                    error!("Failed to append to trades.json: {}", e);
                }

                let trade_msg = json!({ "type": "trade", "trade": trade });
                let _ = ws_tx.send(trade_msg.to_string());
            }
        }
    });
}
