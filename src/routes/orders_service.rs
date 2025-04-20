use std::sync::{Arc, Mutex};

use actix_web::{delete, get, post, web, HttpResponse, Responder, Scope};
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::{
    orders::{Order, TypeOp},
    trading_engine::MatchingEngine,
};

pub fn order_scope() -> Scope {
    web::scope("/zscore")
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
    item: web::Json<Order>,
    tx: web::Data<mpsc::Sender<Order>>,
) -> impl Responder {
    let _ = tx.send(item.into_inner()).await;
    HttpResponse::Ok().body("Order queued")
}

#[delete("/")]
async fn delete_order(
    query: web::Query<DeleteParams>,
    tx: web::Data<mpsc::Sender<Order>>,
) -> impl Responder {
    let del = Order {
        type_op: TypeOp::Delete,
        order_id: query.order_id,
        ..Default::default()
    };
    let _ = tx.send(del).await;
    HttpResponse::Ok().body("Delete queued")
}

#[derive(Deserialize)]
struct DeleteParams {
    order_id: u64,
}
