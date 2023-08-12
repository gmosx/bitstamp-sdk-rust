//! <https://www.bitstamp.net/websocket/v2/>

use std::pin::Pin;

use futures::{stream::SplitSink, StreamExt};
use futures_util::{SinkExt, Stream};
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::{
    api::subscribe::BookEvent,
    types::{Request, Result},
};

pub const DEFAULT_WS_BASE_URL: &str = "wss://ws.bitstamp.net";

/// A WebSocket client for Bitstamp.
pub struct Client {
    sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    // The thread_handle will be dropped when the Client drops.
    #[allow(dead_code)]
    thread_handle: tokio::task::JoinHandle<()>,
    pub broadcast: Option<tokio::sync::broadcast::Sender<String>>,
    pub book_events: Option<Pin<Box<dyn Stream<Item = BookEvent> + Send + Sync>>>,
}

impl Client {
    pub async fn connect(url: &str) -> Result<Self> {
        let (stream, _) = connect_async(url).await?;
        let (sender, receiver) = stream.split();
        let (broadcast_sender, _) = tokio::sync::broadcast::channel::<String>(32);

        let broadcast = broadcast_sender.clone();

        let thread_handle = tokio::spawn(async move {
            let mut receiver = receiver;

            while let Some(result) = receiver.next().await {
                if let Ok(msg) = result {
                    if let Message::Text(string) = msg {
                        tracing::debug!("{string}");
                        if let Err(err) = broadcast_sender.send(string) {
                            tracing::trace!("{err:?}");
                            // Break the while loop so that the receiver handle is dropped
                            // and the task unsubscribes from the summary stream.
                            break;
                        }
                    }
                } else {
                    tracing::error!("{:?}", result);
                }
            }
        });

        Ok(Self {
            sender,
            thread_handle,
            broadcast: Some(broadcast),
            book_events: None,
        })
    }

    pub async fn connect_public() -> Result<Self> {
        let url = format!("{DEFAULT_WS_BASE_URL}/");
        Self::connect(&url).await
    }

    /// Sends a message to the WebSocket.
    pub async fn send<R>(&mut self, req: R) -> Result<()>
    where
        R: Serialize,
    {
        let msg = serde_json::to_string(&req).unwrap();
        tracing::debug!("{msg}");
        self.sender.send(Message::Text(msg.to_string())).await?;

        Ok(())
    }

    /// Performs a remote procedure call.
    pub async fn call<D>(&mut self, event: impl Into<String>, data: D) -> Result<()>
    where
        D: Serialize,
    {
        let req = Request {
            event: event.into(),
            data,
        };

        self.send(req).await
    }
}
