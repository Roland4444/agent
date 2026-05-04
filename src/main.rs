use std::fs;
//          python3 -m http.server 9999

use anyhow::{Context, Result};
use chrono::Local;
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use thirtyfour::prelude::*;
use tokio::time::{Duration, sleep};
use common::*;

use serde_json::json;
use scraper::{Html, Selector};


use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
pub mod http_handler;
//use futures_util::{SinkExt, StreamExt};
pub const PASS_FIELNAME: &str = "pass";
const LOGIN_FILENAME: &str = "login";

const URL_WS_CONNECT: &str = "ws://127.0.0.1:3000/proc";

fn pass() -> Option<String> {
    read_from_file(PASS_FIELNAME)
}

fn login() -> Option<String> {
    read_from_file(LOGIN_FILENAME)
}

fn read_from_file(filename: &str) -> Option<String> {
    let g = fs::read_to_string(filename);
    match g {        Ok(str) => Some(str),        Err(_) => None,}
}

async fn take_screenshot(driver: &WebDriver, base_name: &str) -> Result<String> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("{}_{}.png", base_name, timestamp);
    driver.screenshot(Path::new(&filename)).await?;
    println!("Скриншот сохранён: {}", filename);
    Ok(filename)
}

pub async fn click_collab_simple(driver: &WebDriver) -> Result<()> {
    let by_xpath = By::XPath("//*[text()='Коллабы']");
    let element = driver.find(by_xpath).await?;
    element.click().await?;
    Ok(())
}

pub fn add_magyar(a: u32, b: u32) -> u32 {
    a + b
}

pub async fn click_collab_simple_text(driver: &WebDriver, text: &str) -> Result<()> {
    let condition = format!("//*[text()='{}']", text);
    let by_xpath = By::XPath(condition);
    let element = driver.find(by_xpath).await?;
    element.click().await?;
    Ok(())
}

async fn wait() {
    sleep(Duration::from_secs(30)).await;
}

async fn wait_in_sec(delay: u64) {
    sleep(Duration::from_secs(delay)).await;
}

async fn init_chrome_driver() -> Result<Arc<WebDriver>> {
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--no-sandbox")?;

    // caps.add_arg("--headless=new")?;  // вместо --headless
    caps.add_arg("--disable-blink-features=AutomationControlled")?;
    caps.add_arg("--disable-features=IsolateOrigins,site-per-process")?;
    //   caps.add_arg("--headless")?;                                //
    caps.add_arg("--window-size=1920,1080")?;
    caps.add_arg("--disable-gpu")?; //
    caps.add_arg("--disable-software-rasterizer")?; //    
    caps.add_arg("--disable-dev-shm-usage")?; //
    //  caps.add_arg("--remote-debugging-port=9222")?;              //
    let driver = WebDriver::new("http://localhost:21000", caps).await?;
    Ok(Arc::new(driver))
}

async fn process_item_with_delay(_delay: u64, text_collab: &str, driver: &WebDriver) -> Result<()> {
    click_collab_simple_text(&driver, text_collab).await?;
    scroll_chat_to_bottom(driver).await?;
    wait_in_sec(_delay).await;
    Ok(())
}
/////////////////////////////////////////////
/////////////////////////////////////////////
pub async fn get_message_html_by_chat_and_id(    driver: &WebDriver,    chat_name: &str,    message_id: u64,) -> Result<String> {
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

    Ok(html_str)
}
/////////////////////////////////////////////////
/////////////////////////////////////////////////
async fn scroll_chat_to_bottom(driver: &WebDriver) -> Result<()> {
    println!("=== Начинаем прокрутку чата ===");

    let mut iteration = 0;
    loop {
        iteration += 1;
        let button_selector = By::Css(            ".bx-im-dialog-chat__float-buttons_button, .bx-im-dialog-chat__float-button_icon",        );
        match driver.query(button_selector).wait(Duration::from_secs(2), Duration::from_millis(300)).and_clickable().first().await
        {
            Ok(button) => {
                println!("Iteration:: {}, click button!", iteration);
                if let Err(e) = button.click().await {
                    println!("CLICL FAILED - {}. TRY JS", e);
                    driver.execute("arguments[0].click();", vec![button.to_json()?]).await?;
                }
                sleep(Duration::from_millis(1500)).await;
            }
            Err(_) => {                println!("Button not found! Stop!");                break;            }
        }
        if iteration >= 20 {            println!("REACHED LIMIT ITERATIONS!!!");            break;        }
    }
    println!("SCROLL COMPLETE!");
    Ok(())
}

async fn login_cad(driver: &WebDriver, username: &str, pass: &str) -> Result<()> {
    // Поле логина
    let login_field = driver.query(By::Css("input.b24net-text-input__field[type='text']")).wait(Duration::from_secs(10), Duration::from_millis(500))
        .and_clickable().first().await.context("Поле логина не появилось")?;
    login_field.send_keys(username).await?;

    // Кнопка "Продолжить" (первая)
    let continue_btn = driver
        .query(By::Css(".b24net-login-enter-form__continue-btn")).wait(Duration::from_secs(10), Duration::from_millis(500))
        .and_clickable().first().await.context("Кнопка 'Продолжить' не появилась")?;
    continue_btn.click().await?;

    // Поле пароля
    let password_field = driver
        .query(By::Css("input.b24net-text-input__field[type='password']"))
        .wait(Duration::from_secs(30), Duration::from_millis(500))
        .and_clickable()
        .first()
        .await
        .context("Поле пароля не появилось")?;
    password_field.send_keys(pass).await?;

    // Кнопка "Продолжить" (вторая)
    let submit_btn = driver
        .query(By::Css(".b24net-password-enter-form__continue-btn"))
        .wait(Duration::from_secs(10), Duration::from_millis(500))
        .and_clickable()
        .first()
        .await
        .context("Кнопка 'Продолжить' после пароля не появилась")?;
    submit_btn.click().await?;

    Ok(())
}

const BASE_URL: &str = "https://relits.bitrix24.ru";

pub async fn extract_quoted_text_by_chat_and_message_id(   driver: &WebDriver,    chat_name: &str,    message_id: &str,) -> Result<String> {
    click_collab_simple_text(driver, chat_name).await?;
    sleep(Duration::from_secs(5)).await;
    let msg_selector = By::XPath(&format!("//div[@data-id='{}']", message_id));
    let msg_element = driver.find(msg_selector).await.with_context(|| {  format!("Сообщение с data-id={} не найдено в чате {}", message_id, chat_name ) })?;
    let quote_selector = By::Css(".bx-im-message-quote__text");
    let quote_element = msg_element.find(quote_selector).await.context("Не найден блок цитируемого текста (bx-im-message-quote__text)")?;
    let quoted_text = quote_element.text().await?;
    Ok(quoted_text)
}

async fn send_msg_ws(msg: String) -> () {
    let (mut ws_stream, _) = connect_async("ws://127.0.0.1:3000/ws").await.expect("Не удалось подключиться к серверу");

    ws_stream.send(Message::text(msg)).await.expect("Ошибка отправки сообщения");

    if let Some(Ok(Message::Text(reply))) = ws_stream.next().await {   println!("Ответ сервера: {}", reply); } 
    else {        eprintln!("Сервер не ответил");    }
    ws_stream.close(None).await.ok();
}

pub async fn get_full_info_via_id_and_chat(chat_name: String, message_id: u64) -> Result<QuoteInfo> {
    let (mut ws_stream, _) = connect_async(URL_WS_CONNECT)        .await        .context("Не удалось подключиться к WebSocket")?;

    let req = json!({        "collab": chat_name,        "message_id": message_id,        "type__": "ExtractFull"    });
    let req_bytes = serde_json::to_vec(&req)?;
    ws_stream.send(Message::Binary(req_bytes.into())).await?;

    if let Some(Ok(Message::Text(resp_text))) = ws_stream.next().await {
        let resp: ExtractResp = serde_json::from_str(&resp_text)?;  // сначала разбираем обёртку
        if resp.success {
            if let Some(json_str) = resp.quoted_text {
                let quote_info: QuoteInfo = serde_json::from_str(&json_str)?; // потом внутренность
                println!("Полная информация: {:?}", quote_info);
                return Ok(quote_info);
            } else {
                anyhow::bail!("Ответ не содержит данных");
            }
        } else {
            anyhow::bail!("Ошибка сервера: {}", resp.error.unwrap_or_default());
        }
    }
    anyhow::bail!("Не получен ответ от сервера");
}

pub fn extract_author(input: String) -> Option<String> {
    let fragment = Html::parse_fragment(&input);
    let selector = Selector::parse(".bx-im-chat-title__text").ok()?;
    fragment.select(&selector).next().map(|el| el.text().collect::<String>())
}

pub async fn get_text_via_chat_id_and_id(chat_name: String, message_id: u64) -> Result<String> {
    let (mut ws_stream, _) = connect_async(URL_WS_CONNECT).await.context("Не удалось подключиться к WebSocket")?;
    let request = json!({        "collab": chat_name,        "message_id": message_id, "type__":  "ExtractFull" });
    let request_bytes = serde_json::to_vec(&request)?;
    ws_stream.send(Message::Binary(request_bytes.into())).await?;

    if let Some(Ok(Message::Text(resp_text))) = ws_stream.next().await {
        let resp: ExtractResp = serde_json::from_str(&resp_text)?;
        if resp.success {
            if let Some(text) = resp.quoted_text {                println!("EXTRACTED: {}", text);                return Ok(text);            } 
            else {                anyhow::bail!("Ответ не содержит текста");            }}
        else {            anyhow::bail!("Ошибка сервера: {}", resp.error.unwrap_or_default());        }}
    anyhow::bail!("Не получен ответ от сервера");
}
#[tokio::main]
async fn main() -> Result<()> {
    let test = [PAYMENTS, OWN];
    let iterate = [PAYMENTS, TETRIS, KUIB, OLIVIA, BABEFA, OKLAND, RED, SCANDINAVIA,   POLZ, ZVEZD,  SKY,   OWN,    ];

    const BASE_URL: &str = "https://relits.bitrix24.ru";
    let user_id = 1;
    let driver = init_chrome_driver().await?;

    let profile_url = format!("{}/company/personal/user/{}", BASE_URL, user_id);
    println!("LINK::{}", profile_url);
    driver.goto(&profile_url).await?;

    login_cad( &driver,login().expect("SHIT HAPPENS").as_str(),  pass().expect("SHIT HAPPENS").as_str(),  ).await?;
    wait_in_sec(5).await;
    click_collab_simple(&driver).await?;
    wait_in_sec(1).await;
   // let resp = get_full_info_via_id_and_chat(OKLAND.to_string(), 118782).await;


  //  let HTML = get_message_html_by_chat_and_id(&driver, OKLAND, 120900).await?;

  //  println!("\n\n\nHTML::{}\n\n\n\n\n", HTML);


    let driver_clone = driver.clone();
    let server_handle = tokio::spawn(async move {
        if let Err(e) = http_handler::spawn(driver_clone).await {            eprintln!("Server error: {}", e);        } });

    loop {
        println!("Main thread works...");
        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}

#[cfg(test)]

mod tests {

    //use crate::add_magyar;
    use super::*;
    use common::*; 
    #[test]
    fn test_add() {
        assert_eq!(3, add_magyar(1, 2));
    }

    #[test]
    fn test_read_login() {
        let login_file = "login.txt";
        let login = "rpastushkovb@relits.ru";
        let _ = fs::write(login_file, login);
        let readed = read_from_file(login_file).expect("not found");
        println!("READED:: {}", readed);
        assert_eq!(
            login.to_string(),
            read_from_file(login_file).expect("PANIC")
        );
    }

    #[tokio::test]
    async fn test_send_own_to_websocket() {
        let iterate = [PAYMENTS, TETRIS, KUIB, OLIVIA, BABEFA, OKLAND, RED, SCANDINAVIA,   POLZ, ZVEZD,  SKY,   OWN,    ];
        for item in iterate.iter() {            send_msg_ws(item.to_string()).await;     }
    }

    #[tokio::test]
    async fn test_websocket_extract_quote() {
        let id_old = 118782;
        let id__ = 123820;
        let resp: std::result::Result<String, anyhow::Error> = get_text_via_chat_id_and_id(OKLAND.to_string(), id_old).await;
        match resp {
            Ok(text) => {             println!("EXTRACTED::>>>{}", text)            }
            Err(e) => {                println!("FAILED!, error::{}", e)            }
        }
        let resp2: std::result::Result<String, anyhow::Error> = get_text_via_chat_id_and_id(OKLAND.to_string(), id__).await;
        match resp2 {
            Ok(text) => {                println!("EXTRACTED::>>>{}", text)            }
            Err(e) => {                println!("FAILED!, error::{}", e)            }
        }
    }


const str__: &str  = r#"
<div class="bx-im-message-base__body"><div class="bx-im-message-default__container"><div class="bx-im-message-author-title__container --clickable"><div class="bx-im-chat-title__scope bx-im-chat-title__container"><span class="bx-im-chat-title__content"><!----><span class="    " title="Сергей Музданбаев" style="color: rgb(88, 204, 71);">Сергей Музданбаев</span><!----><!----><!----></span></div></div><div class="bx-im-message-default-content__container bx-im-message-default-content__scope"><div class="bx-im-message-quote --reply --collapsed --clickable" data-context="chat6986/119486"><div class="bx-im-message-quote__wrap"><div class="bx-im-message-quote__name"><div class="bx-im-message-quote__name-text">Артур Сераждинов</div></div><div class="bx-im-message-quote__text">Прошу согласовать материал <br>1.гофра серая 20-ый диаметр-5000м</div><!----></div></div><div class="bx-im-message-default-content__text">Согласовано</div><!----><div class="bx-im-message-default-content__bottom-panel"><!----><div class="bx-im-message-default-content__status-container"><div class="bx-im-message-status__container"><!----><div class="bx-im-message-status__date">11:25</div><!----></div></div></div></div></div><!----><div class="bx-im-reaction-selector__container"><div class="bx-im-reaction-selector__selector"><div class="bx-im-reaction-selector__icon"></div></div></div></div>
"#; 


#[tokio::test]
async fn test_websocket_extract_full_info() {            //                         cargo test test_websocket_extract_full_info -- --nocapture
    let resp = get_full_info_via_id_and_chat(OKLAND.to_string(), 118782).await;
    match resp {
        Ok(qi) => {
            println!("ID: {}", qi.message_id);
           // println!("Автор ответа: {}", qi.message_author);
            println!("Автор цитаты: {}", qi.quoted_author);
            println!("Текст цитаты: {}", qi.quoted_text);
            println!("Текст ответа: {:?}", qi.reply_text);
        }
        Err(e) => eprintln!("Ошибка: {}", e),
    }
}

#[test]           //       cargo test test_extract_author -- --nocapture
fn test_extract_author(){
    let str2__ : &str= r#"<div class="bx-im-message-base__wrap bx-im-message-base__scope --opponent" data-id="120900" data-viewed="true" containerheight="706"><div class="bx-im-message-base__container"><div class="bx-im-message-base__content"><div class="bx-im-message-base__body"><div class="bx-im-message-default__container"><div class="bx-im-message-author-title__container --clickable"><div class="bx-im-chat-title__scope bx-im-chat-title__container"><span class="bx-im-chat-title__content"><!----><span class="bx-im-chat-title__text" title="Сергей Музданбаев" style="color: rgb(88, 204, 71);">Сергей Музданбаев</span><!----><!----><!----></span></div></div><div class="bx-im-message-default-content__container bx-im-message-default-content__scope"><div class="bx-im-message-quote --reply --collapsed --clickable" data-context="chat6986/120850"><div class="bx-im-message-quote__wrap"><div class="bx-im-message-quote__name"><div class="bx-im-message-quote__name-text">Николай Шагов</div></div><div class="bx-im-message-quote__text">Заявка Шагов<br>Тройник канал.рыжий 160×110(45град)×160-18шт<br>Бутылка кан. Рыжая 160&gt;110- 6шт</div><!----></div></div><div class="bx-im-message-default-content__text">Согласовано</div><!----><div class="bx-im-message-default-content__bottom-panel"><!----><div class="bx-im-message-default-content__status-container"><div class="bx-im-message-status__container"><!----><div class="bx-im-message-status__date">08:02</div><!----></div></div></div></div></div><!----><div class="bx-im-reaction-selector__container"><div class="bx-im-reaction-selector__selector"><div class="bx-im-reaction-selector__icon"></div></div></div></div><div class="bx-im-message-context-menu__container bx-im-message-context-menu__scope"><button title="Кликните для открытия меню действий или удерживайте CTRL для цитирования сообщения" class="bx-im-message-context-menu__button"></button></div></div><!----></div></div>"#;
    let etalon_author = "Сергей Музданбаев";
    assert_eq!(etalon_author, extract_author(str2__.to_string()).unwrap_or("".to_string()));
}
}
