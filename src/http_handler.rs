// http_handler.rs
use anyhow::{Context, Result};
use axum::{
    Router,
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use thirtyfour::prelude::*;

// Состояние приложения
pub struct AppState {
    driver: Arc<WebDriver>,
}

#[derive(Debug, Deserialize)]
pub struct ExtractReq {
    pub collab: String,
    pub message_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractResp {
    pub success: bool,
    pub quoted_text: Option<String>,
    pub error: Option<String>,
}

async fn extract_quoted_text(
    driver: &WebDriver,
    chat_name: &str,
    message_id: u64,
) -> Result<String> {
    let condition = format!("//*[text()='{}']", chat_name);
    let by_xpath = By::XPath(condition);
    let element = driver
        .find(by_xpath)
        .await
        .with_context(|| format!("Чат '{}' не найден", chat_name))?;
    element.click().await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let msg_selector = By::XPath(&format!("//div[@data-id='{}']", message_id));
    let msg_element = driver
        .find(msg_selector)
        .await
        .with_context(|| format!("Сообщение с id={} не найдено", message_id))?;

    let quote_selector = By::Css(".bx-im-message-quote__text");
    let quote_element = msg_element
        .find(quote_selector)
        .await
        .context("Блок цитаты не найден")?;

    let text = quote_element.text().await?;
    Ok(text)
}

async fn proc_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(|socket| handle_proc_socket(socket, state))
}

async fn handle_proc_socket(mut socket: WebSocket, state: Arc<AppState>) {
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Binary(bin) => {
                let req: ExtractReq = match serde_json::from_slice(&bin) {
                    Ok(r) => r,
                    Err(e) => {
                        let resp = ExtractResp {
                            success: false,
                            quoted_text: None,
                            error: Some(format!("Invalid JSON: {}", e)),
                        };
                        let _ = socket
                            .send(Message::Text(serde_json::to_string(&resp).unwrap().into()))
                            .await;
                        continue;
                    }
                };
                let result = extract_quoted_text(&state.driver, &req.collab, req.message_id).await;
                let resp = match result {
                    Ok(text) => ExtractResp {
                        success: true,
                        quoted_text: Some(text),
                        error: None,
                    },
                    Err(e) => ExtractResp {
                        success: false,
                        quoted_text: None,
                        error: Some(e.to_string()),
                    },
                };
                let _ = socket
                    .send(Message::Text(serde_json::to_string(&resp).unwrap().into()))
                    .await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                println!("RECEIVED: {}", text);
                if let Err(e) = crate::process_item_with_delay(0, &text, &state.driver).await {
                    eprintln!("Ошибка обработки: {}", e);
                }
                if socket
                    .send(Message::Text(format!("ECHO:: {}", text).into()))
                    .await
                    .is_err()
                {
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
        .route("/proc", get(proc_handler))
        .with_state(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("server run @ ws://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
