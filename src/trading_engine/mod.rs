use std::collections::{BTreeMap, HashMap, VecDeque};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::orders::{Order, Side, TypeOp};

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
pub struct OrderBook {
    buys: BTreeMap<Decimal, VecDeque<Order>>,
    sells: BTreeMap<Decimal, VecDeque<Order>>,
}

impl OrderBook {
    fn new() -> Self {
        Self {
            buys: BTreeMap::new(),
            sells: BTreeMap::new(),
        }
    }

    pub fn process_create(&mut self, order: Order, trades: &mut Vec<Trade>) {
        match order.side {
            Side::Buy => self.match_buy(order, trades),
            Side::Sell => self.match_sell(order, trades),
        }
    }

    fn match_buy(&mut self, mut taker: Order, trades: &mut Vec<Trade>) {
        let sell_prices: Vec<Decimal> = self.sells.keys().cloned().collect();
        for price in sell_prices {
            if taker.amount.is_zero() || price > taker.limit_price {
                break;
            }

            let queue = self.sells.get_mut(&price).unwrap();
            while taker.amount > Decimal::ZERO && queue.front().is_some() {
                let maker = queue.front_mut().unwrap();
                let qty = taker.amount.min(maker.amount);
                trades.push(Trade {
                    buy_order_id: taker.order_id,
                    sell_order_id: maker.order_id,
                    amount: qty,
                    price: maker.limit_price,
                    pair: taker.pair.clone(),
                });
                taker.amount -= qty;
                maker.amount -= qty;

                if maker.amount.is_zero() {
                    queue.pop_front();
                }
            }

            if queue.is_empty() {
                self.sells.remove(&price);
            }
        }

        if taker.amount > Decimal::ZERO {
            self.buys
                .entry(taker.limit_price)
                .or_insert_with(VecDeque::new)
                .push_back(taker);
        }
    }

    fn match_sell(&mut self, mut taker: Order, trades: &mut Vec<Trade>) {
        let mut buy_prices: Vec<Decimal> = self.buys.keys().cloned().collect();
        buy_prices.sort_by(|a, b| b.cmp(a));

        for price in buy_prices {
            if taker.amount.is_zero() || price < taker.limit_price {
                break;
            }

            let queue = self.buys.get_mut(&price).unwrap();
            while taker.amount > Decimal::ZERO && queue.front().is_some() {
                let maker = queue.front_mut().unwrap();
                let qty = taker.amount.min(maker.amount);
                trades.push(Trade {
                    buy_order_id: maker.order_id,
                    sell_order_id: taker.order_id,
                    amount: qty,
                    price: maker.limit_price,
                    pair: taker.pair.clone(),
                });
                taker.amount -= qty;
                maker.amount -= qty;

                if maker.amount.is_zero() {
                    queue.pop_front();
                }
            }

            if queue.is_empty() {
                self.buys.remove(&price);
            }
        }

        if taker.amount > Decimal::ZERO {
            self.sells
                .entry(taker.limit_price)
                .or_insert_with(VecDeque::new)
                .push_back(taker);
        }
    }

    pub fn remove_order(&mut self, order_id: u64, side: Side, price: Decimal) {
        let book = if side == Side::Buy {
            &mut self.buys
        } else {
            &mut self.sells
        };
        if let Some(queue) = book.get_mut(&price) {
            if let Some(pos) = queue.iter().position(|o| o.order_id == order_id) {
                queue.remove(pos);
            }
            if queue.is_empty() {
                book.remove(&price);
            }
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct MatchingEngine {
    pub books: HashMap<String, OrderBook>,
    pub order_index: HashMap<u64, (String, Side, Decimal)>,
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

                self.order_index
                    .insert(order.order_id, (order.pair, order.side, order.limit_price));
            }
            TypeOp::Delete => {
                if let Some((pair, side, price)) = self.order_index.remove(&order.order_id) {
                    if let Some(ob) = self.books.get_mut(&pair) {
                        ob.remove_order(order.order_id, side, price);
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
