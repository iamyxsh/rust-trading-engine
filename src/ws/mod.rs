use std::{sync::Arc, time::Duration};

use actix::{Actor, AsyncContext, StreamHandler};
use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use tokio::sync::broadcast;

pub struct WsSession {
    rx: broadcast::Receiver<String>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let mut rx = self.rx.resubscribe();

        ctx.run_interval(Duration::from_millis(50), move |_, ctx| {
            while let Ok(msg) = rx.try_recv() {
                ctx.text(msg);
            }
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(p)) => ctx.pong(&p),
            Ok(ws::Message::Close(c)) => ctx.close(c),
            _ => (),
        }
    }
}

#[get("/ws")]
async fn ws_index(
    req: HttpRequest,
    stream: web::Payload,
    ws_tx: web::Data<Arc<broadcast::Sender<String>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let rx = ws_tx.subscribe();
    ws::start(WsSession { rx }, &req, stream)
}
