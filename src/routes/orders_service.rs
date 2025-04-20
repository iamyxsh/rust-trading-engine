use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use actix_web::{delete, get, post, web, HttpResponse, Responder, Scope};
use rust_decimal::Decimal;
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::{
    orders::{Order, Side, TypeOp},
    trading_engine::matching_engine::MatchingEngine,
};

pub fn order_scope() -> Scope {
    web::scope("/order")
        .service(get_orders)
        .service(create_order)
        .service(delete_order)
}

#[get("/{account_id}")]
async fn get_orders(
    account_id: web::Path<u64>,
    engine: web::Data<Arc<Mutex<MatchingEngine>>>,
) -> impl Responder {
    let eng = engine.lock().unwrap();

    let result: Vec<Order> = eng.gte_orders_by_account(account_id.into_inner());

    HttpResponse::Ok().json(result)
}

#[post("/")]
async fn create_order(
    item: web::Json<NewOrderRequest>,
    id_counter: web::Data<Arc<AtomicUsize>>,
    tx: web::Data<mpsc::Sender<Order>>,
) -> impl Responder {
    let new_id = id_counter.fetch_add(1, Ordering::SeqCst);

    let order = Order {
        type_op: TypeOp::Create,
        account_id: item.account_id,
        amount: item.amount,
        order_id: new_id as u64,
        pair: item.pair.clone(),
        limit_price: item.limit_price,
        side: item.side.clone(),
    };

    if tx.send(order.clone()).await.is_err() {
        return HttpResponse::InternalServerError().body("Queue error");
    }

    HttpResponse::Ok().json(serde_json::json!({ "order": order }))
}

#[delete("/{order_id}")]
async fn delete_order(
    order_id: web::Path<u64>,
    tx: web::Data<mpsc::Sender<Order>>,
    engine: web::Data<Arc<Mutex<MatchingEngine>>>,
) -> impl Responder {
    let order_id = order_id.into_inner();
    let (pair, side, price, account_id, amount) = {
        let eng = engine.lock().unwrap();
        let orig: &Order = &eng.order_index[&order_id];
        (
            orig.pair.clone(),
            orig.side.clone(),
            orig.limit_price,
            orig.account_id,
            orig.amount,
        )
    };

    let del = Order {
        type_op: TypeOp::Delete,
        order_id,
        pair,
        side,
        limit_price: price,
        account_id,
        amount,
    };

    if tx.send(del).await.is_err() {
        return HttpResponse::InternalServerError().body("Queue error");
    }
    HttpResponse::Ok().body("Delete queued")
}

#[derive(Debug, Deserialize)]
pub struct NewOrderRequest {
    pub account_id: u64,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    pub pair: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub limit_price: Decimal,
    pub side: Side,
}
