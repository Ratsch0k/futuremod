use std::time::Duration;

use async_tungstenite::{WebSocketStream, tungstenite};
use iced::{futures::{self, channel::mpsc}, stream};
use futures::{sink::SinkExt, Stream};
use futures::stream::StreamExt;
use log::*;
use serde::{Serialize, Deserialize};
use tokio::time::Instant;


const BUFFER_TIME: usize = 100;


#[derive(Debug, Clone)]
pub enum Event {
    Connected,
    Disconnected,
    Message(LogRecord),
}

pub enum State {
    Connected(WebSocketStream<async_tungstenite::tokio::ConnectStream>, mpsc::Receiver<Event>, Instant),
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    pub target: String,
    pub message: String,
    pub level: String,
    pub timestamp: String,
    pub plugin: Option<String>
}

pub fn connect(base_address: String) -> impl Stream<Item = Event> {
    stream::channel(
        100,
        |mut output| async move {
            let mut state = State::Disconnected;

            loop {
                match &mut state {
                    State::Disconnected => {
                        match async_tungstenite::tokio::connect_async(
                            format!("ws://{base_address}/log")
                        )
                        .await
                        {
                            Ok((websocket, _)) => {
                                info!("Connected to log websocket");
                                let (_sender, receiver) = mpsc::channel(BUFFER_TIME);
                                let _ = output.send(Event::Connected).await;

                                state = State::Connected(websocket, receiver, Instant::now());
                            }
                            Err(e) => {
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                                warn!("Could not connect to log websocket: {}", e);

                                state = State::Disconnected;
                                let _ = output.send(Event::Disconnected).await;
                            }
                        }
                    }
                    State::Connected(websocket, _input, last_flush) => {
                        let mut fused_websocket = websocket.by_ref().fuse();

                        futures::select! {
                            received = fused_websocket.select_next_some() => {
                                match received {
                                    Ok(tungstenite::Message::Text(message)) => {
                                        match serde_json::from_str::<LogRecord>(message.as_str()) {
                                            Ok(record) => {
                                                let _ = output.feed(Event::Message(record)).await;

                                                let now = Instant::now();
                                                if now.duration_since(*last_flush) >= Duration::from_millis(100) {
                                                    if let Err(err) = output.flush().await {
                                                        warn!("Could not flush pending message: {}", err.to_string());
                                                    }
                                                }
                                            },
                                            Err(e) => {
                                                warn!("Could not parse incoming log record: {:?}", e);
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        warn!("Error occurred while processing log messages: {}", e.to_string());
                                        state = State::Disconnected;
                                        let _ = output.send(Event::Disconnected).await;
                                    },
                                    Ok(_) => (),
                                }
                            },
                            complete => (),
                        }
                    },
                }
            }
        }
    )
}