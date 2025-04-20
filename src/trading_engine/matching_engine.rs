use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::orders::{Order, TypeOp};

use super::orderbook::OrderBook;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    pub buy_order_id: u64,
    pub sell_order_id: u64,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    pub pair: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct MatchingEngine {
    pub books: HashMap<String, OrderBook>,
    pub order_index: HashMap<u64, Order>,
}

impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            books: HashMap::new(),
            order_index: HashMap::new(),
        }
    }

    pub fn process(&mut self, order: Order, trades: &mut Vec<Trade>) {
        match order.type_op {
            TypeOp::Create => {
                let ob = self
                    .books
                    .entry(order.pair.clone())
                    .or_insert_with(OrderBook::new);

                ob.process_create(order.clone(), trades);

                self.order_index.insert(order.order_id, order);
            }
            TypeOp::Delete => {
                if let Some(order) = self.order_index.remove(&order.order_id) {
                    if let Some(ob) = self.books.get_mut(&order.pair) {
                        ob.remove_order(order.order_id, order.side, order.limit_price);
                    }
                }
            }
        }
    }

    pub fn gte_orders_by_account(&self, account_id: u64) -> Vec<Order> {
        let mut result: Vec<Order> = Vec::new();
        for book in self.books.values() {
            for queue in book.buys.values().chain(book.sells.values()) {
                for order in queue {
                    if order.account_id == account_id {
                        result.push(order.clone());
                    }
                }
            }
        }

        return result;
    }
}
