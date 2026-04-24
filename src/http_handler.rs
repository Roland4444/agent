// http_handler.rs
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::Response,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::Result;
use thirtyfour::prelude::*;

// Состояние приложения
pub struct AppState {
    driver: Arc<WebDriver>,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                println!("RECEIVED: {}", text);
                // Вызываем вашу функцию из основного модуля (она должна быть видна)
                if let Err(e) = crate::process_item_with_delay(15, &text, &state.driver).await {
                    eprintln!("Ошибка обработки: {}", e);
                }
                if socket.send(Message::text(format!("ECHO:: {}", text))).await.is_err() {
                    break;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

pub async fn spawn(driver: Arc<WebDriver>) -> Result<()> {
    let state = Arc::new(AppState { driver });
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("server run @ ws://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}