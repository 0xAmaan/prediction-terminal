//! WebSocket route handler
//!
//! Handles WebSocket upgrade and connection management.

use axum::{
    body::Bytes,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use tracing::info;

use crate::AppState;

/// Create WebSocket routes
pub fn routes() -> Router<AppState> {
    Router::new().route("/ws", get(ws_handler))
}

/// WebSocket upgrade handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    info!("WebSocket upgrade request received");
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle an established WebSocket connection
async fn handle_socket(socket: WebSocket, state: AppState) {
    // Convert axum WebSocket to tokio-tungstenite compatible stream
    let (mut sender, mut receiver) = socket.split();

    // Create channels for bridging between axum and our handler
    let (tx, rx) = tokio::sync::mpsc::channel::<tokio_tungstenite::tungstenite::Message>(100);
    let (response_tx, mut response_rx) =
        tokio::sync::mpsc::channel::<tokio_tungstenite::tungstenite::Message>(100);

    // Task: Forward messages from axum receiver to our channel
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            let tungstenite_msg = match msg {
                Message::Text(text) => {
                    tokio_tungstenite::tungstenite::Message::Text(text.to_string().into())
                }
                Message::Binary(data) => {
                    tokio_tungstenite::tungstenite::Message::Binary(data.to_vec().into())
                }
                Message::Ping(data) => {
                    tokio_tungstenite::tungstenite::Message::Ping(data.to_vec().into())
                }
                Message::Pong(data) => {
                    tokio_tungstenite::tungstenite::Message::Pong(data.to_vec().into())
                }
                Message::Close(_) => {
                    break;
                }
            };

            if tx.send(tungstenite_msg).await.is_err() {
                break;
            }
        }
    });

    // Task: Forward messages from our response channel to axum sender
    let send_task = tokio::spawn(async move {
        while let Some(msg) = response_rx.recv().await {
            let axum_msg = match msg {
                tokio_tungstenite::tungstenite::Message::Text(text) => {
                    Message::Text(text.to_string().into())
                }
                tokio_tungstenite::tungstenite::Message::Binary(data) => {
                    Message::Binary(Bytes::from(data.to_vec()))
                }
                tokio_tungstenite::tungstenite::Message::Ping(data) => {
                    Message::Ping(Bytes::from(data.to_vec()))
                }
                tokio_tungstenite::tungstenite::Message::Pong(data) => {
                    Message::Pong(Bytes::from(data.to_vec()))
                }
                tokio_tungstenite::tungstenite::Message::Close(_) => {
                    break;
                }
                tokio_tungstenite::tungstenite::Message::Frame(_) => continue,
            };

            if sender.send(axum_msg).await.is_err() {
                break;
            }
        }
    });

    // Create a bridge stream that implements the traits needed by our handler
    let bridge = BridgeStream { rx, tx: response_tx };

    // Handle the connection using our WebSocketState
    state.ws_state.handle_connection(bridge).await;

    // Clean up tasks
    recv_task.abort();
    send_task.abort();
}

/// Bridge stream that adapts between channels and our handler's expected interface
struct BridgeStream {
    rx: tokio::sync::mpsc::Receiver<tokio_tungstenite::tungstenite::Message>,
    tx: tokio::sync::mpsc::Sender<tokio_tungstenite::tungstenite::Message>,
}

impl futures_util::Stream for BridgeStream {
    type Item = Result<tokio_tungstenite::tungstenite::Message, tokio_tungstenite::tungstenite::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.rx.poll_recv(cx) {
            std::task::Poll::Ready(Some(msg)) => std::task::Poll::Ready(Some(Ok(msg))),
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl futures_util::Sink<tokio_tungstenite::tungstenite::Message> for BridgeStream {
    type Error = tokio_tungstenite::tungstenite::Error;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn start_send(
        self: std::pin::Pin<&mut Self>,
        item: tokio_tungstenite::tungstenite::Message,
    ) -> Result<(), Self::Error> {
        let _ = self.tx.try_send(item);
        Ok(())
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}
