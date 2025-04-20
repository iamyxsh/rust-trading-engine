use actix_web::{get, web, HttpResponse, Responder, Scope};
use serde::Deserialize;

use crate::{file_io, trading_engine::matching_engine::Trade};

pub fn trades_scope() -> Scope {
    web::scope("/trades").service(get_trades)
}

#[get("/")]
async fn get_trades(query: web::Query<TradesQuery>) -> impl Responder {
    let mut trades: Vec<Trade> = match file_io::read("./jsons/trades.json") {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().body("Could not read trades.json"),
    };

    if let Some(ref pair) = query.pair {
        trades.retain(|t| &t.pair == pair);
    }

    trades.sort_by(|a, b| {
        let cmp = match query.sort.as_str() {
            "amount" => a.amount.cmp(&b.amount),

            _ => a.price.cmp(&b.price),
        };
        if query.order.eq_ignore_ascii_case("desc") {
            cmp.reverse()
        } else {
            cmp
        }
    });

    let start = query.skip;

    let trades = if query.limit == 0 {
        trades.into_iter().skip(start).collect::<Vec<_>>()
    } else {
        trades
            .into_iter()
            .skip(start)
            .take(query.limit)
            .collect::<Vec<_>>()
    };

    HttpResponse::Ok().json(trades)
}

#[derive(Deserialize, Default)]
pub struct TradesQuery {
    pub pair: Option<String>,
    #[serde(default = "default_sort")]
    pub sort: String,

    #[serde(default = "default_order")]
    pub order: String,

    #[serde(default)]
    pub skip: usize,
    #[serde(default)]
    pub limit: usize,
}

fn default_sort() -> String {
    "price".to_string()
}

fn default_order() -> String {
    "asc".to_string()
}
