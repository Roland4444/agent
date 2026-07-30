// http_handler.rs
use anyhow::{Context, Result, bail};
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
use common::*;
use tokio::time::{Duration, sleep};
use crate::extract_author;
use crate::scroll_chat_to_bottom;

// Состояние приложения
pub struct AppState {  driver: Arc<WebDriver>,}


async fn wait_in_sec(delay: u64) {
    sleep(Duration::from_secs(delay)).await;
}


pub async fn click_collab_simple_text(driver: &WebDriver, text: &str) -> Result<()> {
    let condition = format!("//*[text()='{}']", text);
    let by_xpath = By::XPath(condition);
    let element = driver.find(by_xpath).await?;
    element.click().await?;
    Ok(())
}

// Удалите или закомментируйте строку:
// use crate::extract_author;
pub async fn extract_quote_info_by_chat_and_message_id22(
    driver: &WebDriver,
    chat_name: &str,
    message_id: u64,
) -> Result<QuoteInfo> {
    // 1. Открываем чат
    let condition = format!("//*[text()='{}']", chat_name);
    let element = driver
        .find(By::XPath(&condition))
        .await
        .with_context(|| format!("Чат '{}' не найден", chat_name))?;
    element.click().await?;
    sleep(Duration::from_secs(2)).await;
    println!("\n\nSCROLL CHAT {} TO DOWN\n\n", chat_name);
    let _ = scroll_chat_to_bottom(driver).await;
    // 2. Захват фокуса: клик по элементу сообщения (автор или контейнер)
    let focus_selector = ".bx-im-message-list-author-group__container";
    if let Ok(elem) = driver.find(By::Css(focus_selector)).await {

        element.scroll_into_view().await?;
        driver.execute("arguments[0].click();", vec![element.to_json()?]).await?;
        //elem.click().await?;
    } else {
        // fallback: клик по любому сообщению
        if let Ok(any_msg) = driver.find(By::Css(".bx-im-message")).await {
            any_msg.click().await?;
        } else {
            driver.execute("document.body.click();", vec![]).await?;
        }
    }

    // 3. Эмуляция PageUp для прокрутки истории вверх
    let msg_selector = By::XPath(&format!("//div[@data-id='{}']", message_id));
    let max_scrolls = 1000;
    let mut found = false;

    for x in 0..max_scrolls {
        if driver.find(msg_selector.clone()).await.is_ok() {
            found = true;
            break;
        }
        println!("COUNTER::{}\n\n", x);
        // Отправляем PageUp активному элементу (после клика фокус на сообщениях)
        driver
            .execute(
                r#"
                let active = document.activeElement;
                if (active) {
                    let evt = new KeyboardEvent('keydown', { key: 'PageUp', code: 'PageUp', keyCode: 33, which: 33, bubbles: true });
                    active.dispatchEvent(evt);
                }
                "#,
                vec![],
            )
            .await?;
        sleep(Duration::from_millis(300)).await;
    }

    if !found {
        bail!(
            "Сообщение с id={} не найдено после {} нажатий PageUp",
            message_id,
            max_scrolls
        );
    }

    // 4. Парсинг (оригинальная логика)
    let msg_element = driver.find(msg_selector).await?;

    let reply_text = if let Ok(el) = msg_element
        .find(By::Css(".bx-im-message-default-content__text"))
        .await
    {
        el.text().await.ok()
    } else {
        None
    };

    let quoted_author = msg_element
        .find(By::Css(".bx-im-message-quote__name-text"))
        .await
        .context("Не найден автор цитаты")?
        .text()
        .await?;

    let quoted_text = msg_element
        .find(By::Css(".bx-im-message-quote__text"))
        .await
        .context("Текст цитаты не найден")?
        .text()
        .await?;

    Ok(QuoteInfo {
        reply_text,
        quoted_author,
        quoted_text,
        message_id: message_id.to_string(),
    })
}


pub async fn extract_quote_info_by_chat_and_message_id(
    driver: &WebDriver,
    chat_name: &str,
    message_id: u64,
) -> Result<QuoteInfo> {
    // Открываем чат
    let condition = format!("//*[text()='{}']", chat_name);
    let element = driver
        .find(By::XPath(&condition))
        .await
        .with_context(|| format!("Чат '{}' не найден", chat_name))?;
    element.click().await?;
    sleep(Duration::from_secs(2)).await;

    // Прокручиваем вниз до конца (подгружаем последние сообщения)
    scroll_chat_to_bottom(driver).await?;

    // Находим контейнер прокрутки – используем match, чтобы избежать async в замыкании
    let container = match driver
        .find(By::Css(".bx-im-dialog-chat__scroll-container"))
        .await
    {
        Ok(c) => c,
        Err(_) => driver
            .find(By::Css(".bx-im-dialog-chat__block"))
            .await
            .context("Не найден контейнер прокрутки чата")?,
    };

    let msg_selector = By::XPath(&format!("//div[@data-id='{}']", message_id));
    let max_scrolls = 1500;
    let mut found = false;

    for step in 0..max_scrolls {
        if driver.find(msg_selector.clone()).await.is_ok() {
            found = true;
            println!("Сообщение {} найдено после {} шагов", message_id, step);
            break;
        }

        // Прокручиваем контейнер вверх (прямая манипуляция scrollTop)
        driver
            .execute("arguments[0].scrollTop -= 400;", vec![container.to_json()?])
            .await?;

        if step % 100 == 0 {
            println!("Прокрутка вверх, шаг {}", step);
        }
        sleep(Duration::from_millis(200)).await;
    }

    if !found {
        bail!(
            "Сообщение с id={} не найдено после {} шагов",
            message_id,
            max_scrolls
        );
    }

    let msg_element = driver.find(msg_selector).await?;

    let reply_text = if let Ok(el) = msg_element
        .find(By::Css(".bx-im-message-default-content__text"))
        .await
    {
        el.text().await.ok()
    } else {
        None
    };

    let quoted_author = msg_element
        .find(By::Css(".bx-im-message-quote__name-text"))
        .await
        .context("Не найден автор цитаты")?
        .text()
        .await?;

    let quoted_text = msg_element
        .find(By::Css(".bx-im-message-quote__text"))
        .await
        .context("Текст цитаты не найден")?
        .text()
        .await?;

    Ok(QuoteInfo {
        message_id: message_id.to_string(),
        quoted_author,
        quoted_text,
        reply_text,
    })
}
pub async fn extract_quote_info_by_chat_and_message_id__(driver: &WebDriver,chat_name: &str,message_id: u64,) -> Result<QuoteInfo> {

    let condition = format!("//*[text()='{}']", chat_name);
    let by_xpath = By::XPath(condition);
    let element = driver.find(by_xpath).await
        .with_context(|| format!("Чат '{}' не найден", chat_name))?;
    element.click().await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;



    let msg_selector = By::XPath(&format!("//div[@data-id='{}']", message_id));
    let msg_element = driver.find(msg_selector).await
        .with_context(|| format!("Сообщение с id={} не найдено", message_id))?;


    let outer_html = driver.execute("return arguments[0].outerHTML;", vec![msg_element.to_json()?]).await?;
    let html_value = outer_html.json(); // &Value
    let html_str = html_value.as_str().unwrap_or("").to_string();
    println!("MSG::{}", html_str.to_string());
    let reply_text = if let Ok(el) = msg_element.find(By::Css(".bx-im-message-default-content__text")).await {
        el.text().await.ok()} 
    else {        None    };

    let quoted_author = msg_element.find(By::Css(".bx-im-message-quote__name-text")).await
        .context("Не найден автор цитаты")?
        .text()
        .await?;

    let quoted_text = msg_element.find(By::Css(".bx-im-message-quote__text")).await
        .context("Текст цитаты не найден")?
        .text()
        .await?;

    Ok(QuoteInfo {reply_text,quoted_author,quoted_text,message_id:message_id.to_string(),})
}
// pub async fn extract_quote_info_by_chat_and_message_id2(    driver: &WebDriver,    chat_name: &str,    message_id: &str,) -> Result<QuoteInfo> {
//     click_collab_simple_text(driver, chat_name).await?;
//     wait_in_sec(3).await;
//     let msg_selector = By::XPath(&format!("//div[@data-id='{}']", message_id));
//     let msg_element = driver.find(msg_selector).await.with_context(|| {     format!("Сообщение с data-id={} не найдено в чате {}", message_id, chat_name)})?;

//     let author_selector = By::Css(".bx-im-message-author-title__container .bx-im-chat-title__text");
//     let message_author = "NONAME";///..msg_element.find(author_selector).await.context("Не найден автор сообщения")?.text().await?;

//     let reply_text = if let Ok(el) = msg_element.find(By::Css(".bx-im-message-default-content__text")).await {        el.text().await.ok()    } 
//     else {        None    };    // Автор цитируемого сообщения
//     let quoted_author = msg_element.find(By::Css(".bx-im-message-quote__name-text")).await.context("Не найден автор цитаты")?.text().await?;
//     let quoted_text = msg_element.find(By::Css(".bx-im-message-quote__text")).await.context("Не найден текст цитаты")?.text().await?;

//     Ok(QuoteInfo {message_id.to_string(),        message_author.to_string(),        quoted_author,        quoted_text,        reply_text,    })
// }





async fn extract_quoted_text(   driver: &WebDriver,   chat_name: &str,    message_id: u64,) -> Result<String> {
    let condition = format!("//*[text()='{}']", chat_name);
    let by_xpath = By::XPath(condition);
    let element = driver.find(by_xpath).await.with_context(|| format!("Чат '{}' не найден", chat_name))?;
    element.click().await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let msg_selector = By::XPath(&format!("//div[@data-id='{}']", message_id));
    let msg_element = driver.find(msg_selector).await.with_context(|| format!("Сообщение с id={} не найдено", message_id))?;

    let quote_selector = By::Css(".bx-im-message-quote__text");
    let quote_element = msg_element.find(quote_selector).await.context("Блок цитаты не найден")?;
    let text = quote_element.text().await?;
    Ok(text)
}

async fn proc_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {ws.on_upgrade(|socket| handle_proc_socket(socket, state))}


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

                match req.type__ {
                    TypeOperation::ExtractSimple => {
                        println!("\n\n\nEXTRACT SIMPLE!\n\n\n\n");
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
                    TypeOperation::ExtractFull => {
                        println!("\n\n\nEXTRACT FULL!\n\n\n\n");

                        let result = extract_quote_info_by_chat_and_message_id(
                            &state.driver,
                            &req.collab,
                            req.message_id,
                        ).await;
                        match result {
                            Ok(quote_info) => {
                                // Сериализуем полную структуру в JSON и кладём в quoted_text
                                let json_str = serde_json::to_string(&quote_info).unwrap();
                                let resp = ExtractResp {
                                    success: true,
                                    quoted_text: Some(json_str),
                                    error: None,
                                };
                                let _ = socket
                                    .send(Message::Text(serde_json::to_string(&resp).unwrap().into()))
                                    .await;
                            }
                            Err(e) => {
                                let resp = ExtractResp {
                                    success: false,
                                    quoted_text: None,
                                    error: Some(e.to_string()),
                                };
                                let _ = socket
                                    .send(Message::Text(serde_json::to_string(&resp).unwrap().into()))
                                    .await;
                            }
                        }
                    }
                }
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
                if socket.send(Message::Text(format!("ECHO:: {}", text).into())).await.is_err(){ break; }
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



    // let example = r#"
    //     <div class="bx-im-message-base__wrap bx-im-message-base__scope --opponent" data-id="120900" data-viewed="true" containerheight="706">
    //         <div class="bx-im-message-base__container">
    //             <div class="bx-im-message-base__content">
    //                 <div class="bx-im-message-base__body">
    //                     <div class="bx-im-message-default__container">
    //                         <div class="bx-im-message-author-title__container --clickable">
    //                             <div class="bx-im-chat-title__scope bx-im-chat-title__container">
    //                                 <span class="bx-im-chat-title__content"><!---->
    //                                     <span class="bx-im-chat-title__text" title="Сергей Музданбаев" style="color: rgb(88, 204, 71);">
    //                                         Сергей Музданбаев
    //                                     </span>
    //                                 </span>
    //                             </div>
    //                         </div>
    //                         <div class="bx-im-message-default-content__container bx-im-message-default-content__scope">
    //                             <div class="bx-im-message-quote --reply --collapsed --clickable" data-context="chat6986/120850">
    //                                 <div class="bx-im-message-quote__wrap">
    //                                     <div class="bx-im-message-quote__name">
    //                                         <div class="bx-im-message-quote__name-text">Николай Шагов
    //                                         </div>
    //                                     </div>
    //                                     <div class="bx-im-message-quote__text">Заявка Шагов<br>Тройник канал.рыжий 160×110(45град)×160-18шт<br>Бутылка кан. Рыжая 160&gt;110- 6шт
    //                                     </div>
    //                                 </div>
    //                             </div>
    //                         <div class="bx-im-message-default-content__text">Согласовано</div>

    //                         <div class="bx-im-message-default-content__bottom-panel">
    //                             <div class="bx-im-message-default-content__status-container">
    //                                 <div class="bx-im-message-status__container">
    //                                     <div class="bx-im-message-status__date">08:02
    //                                     </div><!---->
    //                                 </div>
    //                             </div>
    //                         </div>
    //                     </div>
    //                 </div>
    //             <div class="bx-im-reaction-selector__container">
    //                 <div class="bx-im-reaction-selector__selector">
    //                     <div class="bx-im-reaction-selector__icon">
    //                     </div>
    //                 </div>
    //             </div>
    //         </div>
    //     <div class="bx-im-message-context-menu__container bx-im-message-context-menu__scope">
    //     <button title="Кликните для открытия меню действий или удерживайте CTRL для цитирования сообщения" class="bx-im-message-context-menu__button">
    //     "#;