use async_stream::stream;
use serde::{Deserialize, Serialize};

use crate::client::Client;

pub const SUBSCRIBE_EVENT: &str = "bts:subscribe";

#[derive(Default, Serialize)]
pub struct SubscribeData {
    channel: String,
}

impl SubscribeData {
    pub fn new(channel: &str) -> Self {
        Self {
            channel: channel.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BookEvent {
    pub data: BookData,
}

#[derive(Debug, Deserialize)]
pub struct BookData {
    pub timestamp: String,
    pub microtimestamp: String,
    pub bids: Vec<(String, String)>,
    pub asks: Vec<(String, String)>,
}

impl Client {
    // <https://assets.bitstamp.net/static/webapp/examples/order_book_v2.3610acefe104f01a8dcd4fda1cb88403f368526b.html>
    pub async fn subscribe_live_orderbook(&mut self, symbol: &str) {
        let channel = format!("order_book_{symbol}");

        self.call(SUBSCRIBE_EVENT, SubscribeData { channel })
            .await
            .expect("cannot send request");

        let mut messages_receiver = self
            .broadcast
            .clone()
            .expect("client not connected")
            .subscribe();

        let depth_events = stream! {
            while let Ok(msg) = messages_receiver.recv().await {
                if let Ok(msg) = serde_json::from_str::<BookEvent>(&msg) {
                    yield msg
                }
            }
        };

        let depth_events = Box::pin(depth_events);

        self.book_events = Some(depth_events);
    }
}
