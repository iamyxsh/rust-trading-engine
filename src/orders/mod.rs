use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;
use std::primitive::str;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TypeOp {
    Create,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub type_op: TypeOp,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub account_id: u64,

    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub order_id: u64,
    pub pair: String,

    #[serde(with = "rust_decimal::serde::str")]
    pub limit_price: Decimal,

    pub side: Side,
}

impl Default for Side {
    fn default() -> Self {
        Side::Buy
    }
}

impl Default for TypeOp {
    fn default() -> Self {
        TypeOp::Create
    }
}
