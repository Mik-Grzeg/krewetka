//! Simple websocket client.

use std::{io, thread};

use actix_web::web::Bytes;
use awc::ws;
use futures_util::{SinkExt as _, StreamExt as _};
use tokio::{select, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
use serde_json::Value;

#[actix_web::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting echo WebSocket client");

    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let mut cmd_rx = UnboundedReceiverStream::new(cmd_rx);

    // run blocking terminal input reader on separate thread
    let input_thread = thread::spawn(move || loop {
        let mut cmd = String::with_capacity(32);

        if io::stdin().read_line(&mut cmd).is_err() {
            log::error!("error reading line");
            return;
        }

        cmd_tx.send(cmd).unwrap();
    });

    let (res, mut ws) = awc::Client::new()
        .ws("ws://127.0.0.1:8080/throughput_ws")
        .connect()
        .await
        .unwrap();

    log::debug!("response: {res:?}");
    log::info!("connected; server will echo messages sent");

    loop {
        select! {
            Some(msg) = ws.next() => {
                match msg {
                    Ok(ws::Frame::Text(txt)) => {
                        log::info!("Raw response: {txt:?}");
                        let txt: &str = std::str::from_utf8(&*txt).unwrap();
                        let v: Value = serde_json::from_str(txt).unwrap();
                        log::info!("Server: {v}")
                    }

                    Ok(ws::Frame::Ping(_)) => {
                        // respond to ping probes
                        ws.send(ws::Message::Pong(Bytes::new())).await.unwrap();
                        log::info!("Mega ping pong")
                    }

                    _ => {}
                }
            }

            Some(cmd) = cmd_rx.next() => {
                if cmd.is_empty() {
                    continue;
                }

                ws.send(ws::Message::Text(cmd.into())).await.unwrap();
            }

            else => break
        }
    }

    input_thread.join().unwrap();
}
