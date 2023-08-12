use bitstamp_ws::client::Client;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let mut client = Client::connect_public().await.expect("cannot connect");

    client.subscribe_live_orderbook("btcusd").await;

    let mut book_events = client.book_events.unwrap();

    while let Some(msg) = book_events.next().await {
        dbg!(&msg);
    }
}
