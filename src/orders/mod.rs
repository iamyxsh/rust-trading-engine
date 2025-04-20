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

impl Order {
    pub fn new(
        type_op: TypeOp,
        account_id: u64,
        amount: impl Into<Decimal>,
        order_id: u64,
        pair: String,
        limit_price: impl Into<Decimal>,
        side: Side,
    ) -> Self {
        Self {
            type_op,
            account_id,
            amount: amount.into(),
            order_id,
            pair: pair.into(),
            limit_price: limit_price.into(),
            side,
        }
    }
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
