use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use std::time::Duration;

const URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";

#[tokio::main]
async fn main() {
    println!("Testing Polymarket WebSocket connection...");
    println!("URL: {}", URL);

    println!("\n1. Attempting connection...");
    let start = std::time::Instant::now();

    match tokio::time::timeout(
        Duration::from_secs(15),
        connect_async(URL)
    ).await {
        Ok(Ok((ws, response))) => {
            println!("✓ Connected in {:?}", start.elapsed());
            println!("  Response status: {:?}", response.status());

            let (mut write, mut read) = ws.split();

            // Super Bowl 2026 - Buffalo Bills YES (more likely to have activity)
            let token = "19740329944962592380580142050369523795065853055987745520766432334608119837023";
            let subscribe_msg = format!(r#"{{"assets_ids":["{}"],"type":"market"}}"#, token);

            println!("\n2. Sending subscription: {}", &subscribe_msg[..100.min(subscribe_msg.len())]);

            if let Err(e) = write.send(Message::Text(subscribe_msg.into())).await {
                println!("✗ Failed to send subscription: {}", e);
                return;
            }
            println!("✓ Subscription sent");

            // Spawn ping task
            let (ping_tx, mut ping_rx) = tokio::sync::mpsc::channel::<()>(1);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(10));
                loop {
                    interval.tick().await;
                    if ping_tx.send(()).await.is_err() {
                        break;
                    }
                }
            });

            println!("\n3. Waiting for messages (60 seconds total, pinging every 10s)...");
            let test_start = std::time::Instant::now();
            let mut msg_count = 0;

            while test_start.elapsed() < Duration::from_secs(60) {
                tokio::select! {
                    msg = read.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                msg_count += 1;
                                let preview = if text.len() > 300 {
                                    format!("{}...", &text[..300])
                                } else {
                                    text.to_string()
                                };
                                println!("  [{:>3}] {:>5.1}s Text: {}", msg_count, test_start.elapsed().as_secs_f64(), preview);
                            }
                            Some(Ok(Message::Ping(data))) => {
                                println!("  [PING] Server ping received");
                                if let Err(e) = write.send(Message::Pong(data)).await {
                                    println!("  Warning: Failed to send pong: {}", e);
                                }
                            }
                            Some(Ok(Message::Pong(_))) => {
                                // Expected response to our ping
                            }
                            Some(Ok(Message::Close(frame))) => {
                                println!("  [CLOSE] Close frame: {:?}", frame);
                                break;
                            }
                            Some(Ok(_)) => {}
                            Some(Err(e)) => {
                                println!("  [ERROR] {}", e);
                                break;
                            }
                            None => {
                                println!("  [END] Stream ended");
                                break;
                            }
                        }
                    }
                    _ = ping_rx.recv() => {
                        // Send ping
                        if let Err(e) = write.send(Message::Text("PING".to_string().into())).await {
                            println!("  [PING] Failed to send: {}", e);
                            break;
                        }
                    }
                }
            }

            println!("\nTest complete! Received {} messages in {:.1}s", msg_count, test_start.elapsed().as_secs_f64());
        }
        Ok(Err(e)) => {
            println!("✗ Connection failed after {:?}: {}", start.elapsed(), e);
        }
        Err(_) => {
            println!("✗ Connection timed out after {:?}", start.elapsed());
        }
    }
}
