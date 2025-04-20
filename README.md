# Order Matching Engine Service

A high-performance order matching engine built in Rust using Actix-Web, Tokio, and an in-memory orderbook. The service supports RESTful HTTP APIs for creating, deleting, and querying orders and trades, automatic incremental ID assignment, background processing via an in-process queue, real-time WebSocket broadcasts of events, and Docker containerization.

## Features

- **Order Processing**

  - `POST /orders` - Create a new order with automatic incremental `order_id`.
  - `DELETE /orders/{id}` - Cancel an existing order; logs full delete record in `orders.json`.
  - Background worker using Tokio `mpsc` channel to append orders to `orders.json`, update in-memory matching engine, write snapshots to `orderbook.json`, append trades to `trades.json`.

- **Query Endpoints**

  - `GET /orders/account/{account_id}` - Retrieve all open orders for a specific account.
  - `GET /trades` - Fetch historical trades with optional query parameters:
    - `pair` (e.g. `BTC/USDC`)
    - `sort`: `price` (default) or `amount`
    - `order`: `asc` (default) or `desc`
    - `skip`: number of records to skip
    - `limit`: max number of records returned

- **WebSocket Real-Time Updates**

  - `GET /ws` - Clients can connect to receive live JSON events:
    - **`order_create`** / **`order_delete`**: when an order is created or canceled.
    - **`trade`**: emitted for each executed trade, includes `pair`, `buy_order_id`, `sell_order_id`, `amount`, `price`.
    - **`orderbook_snapshot`**: full latest orderbook state after each match.

- **Persistence**

  - `orders.json` - Logs every `CREATE` and `DELETE` action with full order details.
  - `orderbook.json` - Overwritten after each order processed, containing current orderbook map.
  - `trades.json` - Appends JSON entries for each executed trade.

- **Auto-Increment IDs**

  - On startup, scans existing `orders.json` for highest `order_id` to initialize the atomic counter.
  - Ensures new orders receive unique, incremental IDs.

- **Docker & Docker Compose**
  - Multi-stage `Dockerfile` to build and run the Rust service in a slim container.
  - `docker-compose.yml` to spin up the service with a mounted `jsons/` directory for persistent data.

## Prerequisites

- Rust >= 1.74
- Docker & Docker Compose (if containerization is desired)

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/iamyxsh/rust-trading-engine
   cd matching-engine
   ```
2. Build and run locally:
   ```bash
   cargo run --release
   ```
3. The service listens on port **5050** by default.

## Usage

### Create Order

```http
POST /orders
Content-Type: application/json

{
  "account_id": 42,
  "amount": "0.25000",
  "pair": "BTC/USDC",
  "limit_price": "60500.00",
  "side": "BUY"
}
```

Response:

```json
{ "order_id": 1001 }
```

### Delete Order

```http
DELETE /orders/1001
```

### Get Orders by Account

```http
GET /orders/account/42
```

### Query Trades

```http
GET /trades?pair=BTC/USDC&sort=amount&order=desc&skip=0&limit=10
```

### WebSocket Stream

Connect to:

```
ws://localhost:5050/ws
```

Receive events:

```json
{ "type": "order_create", "order": { ... } }
{ "type": "trade",        "trade": { ... } }
{ "type": "orderbook_snapshot", "books": { ... } }
```

## Postman Collection

A ready-to-import Postman collection is available in `./docs/postman_collection.json`. It includes predefined requests for all REST endpoints (`/orders`, `/trades`, etc.) and WebSocket tests.

## Docker

This service uses **cargo-chef** to dramatically speed up Docker image rebuilds by caching Rust dependency compilation.

1. **Build & run** with Docker Compose:
   ```bash
   docker-compose up --build
   ```

The service will be available at `http://localhost:5050` and persists JSON data in `./jsons/`.
