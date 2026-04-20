use anyhow::{Result, Context};
use thirtyfour::prelude::*;
use tokio::time::{Duration, sleep};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

const PAYMENTS: &str = "Платежи";
const OLIVIA: &str = "ОЛИВИЯ МАКСАКОВА";
const BABEFA: &str = "ЖК Бабефа";
const OKLAND: &str = "ОКЛАНД РЫБАЦКАЯ";
const RED: &str = "РЭД Грузинская";
const TETRIS: &str = "ЖК Тетрис на Керченской";
const SCANDINAVIA: &str = "Скандинавия - Моздокская";
const KUIB: &str = "Куйбышева";
const POLZ: &str = "Ползунова";
const ZVEZD: &str = "Звездная";
const SKY: &str = "СКАЙ ИГАРСКАЯ";

const OWN: &str = "OWN";

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

async fn init_chrome_driver() -> Result<WebDriver> {
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    let driver = WebDriver::new("http://localhost:21000", caps).await?;
    Ok(driver)
}

async fn scroll_down_current_collab() -> Result<()> {
    Ok(())
}

async fn process_item_with_delay(delay: u64, text_collab: &str, driver: &WebDriver) -> Result<()> {
    click_collab_simple_text(&driver, text_collab).await?;
    ///works!
    scroll_chat_to_bottom(driver).await?;
    wait_in_sec(15).await;
    Ok(())
}

async fn scroll_chat_to_bottom(driver: &WebDriver) -> Result<()> {
    println!("=== Начинаем прокрутку чата ===");

    let mut iteration = 0;
    loop {
        iteration += 1;
        let button_selector = By::Css(
            ".bx-im-dialog-chat__float-buttons_button, .bx-im-dialog-chat__float-button_icon",
        );
        match driver
            .query(button_selector)
            .wait(Duration::from_secs(2), Duration::from_millis(300))
            .and_clickable()
            .first()
            .await
        {
            Ok(button) => {
                println!("Iteration:: {}, click button!", iteration);
                if let Err(e) = button.click().await {
                    println!("CLICL FAILED - {}. TRY JS", e);
                    driver
                        .execute("arguments[0].click();", vec![button.to_json()?])
                        .await?;
                }
                sleep(Duration::from_millis(1500)).await;
            }
            Err(_) => {
                println!("Button not found! Stop!");
                break;
            }
        }
        if iteration >= 20 {
            println!("REACHED LIMIT ITERATIONS!!!");
            break;
        }
    }
    println!("SCROLL COMPLETE!");
    Ok(())
}


async fn connect() -> Result<()> {
    let (mut websocket_stream, _) = connect_async("ws://127.0.0.1:3000/ws")
        .await
        .context("Не удалось подключиться к WebSocket")?;

    websocket_stream
        .send(Message::text("Привет, сервер!"))
        .await
        .context("Не удалось отправить сообщение")?;

    if let Some(Ok(Message::Text(reply))) = websocket_stream.next().await {
        println!("Ответ от сервера: {}", reply);
    }

    websocket_stream.close(None).await.ok();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let test = [PAYMENTS, OWN];
    let iterate = [
        PAYMENTS,
        TETRIS,
        KUIB,
        OLIVIA,
        BABEFA,
        OKLAND,
        RED,
        SCANDINAVIA,
        POLZ,
        ZVEZD,
        SKY,
    ];
    const BASE_URL: &str = "https://relits.bitrix24.ru";
    let user_id = 1;
    let driver = init_chrome_driver().await?;

    let profile_url = format!("{}/company/personal/user/{}", BASE_URL, user_id);
    println!("LINK::{}", profile_url);
    driver.goto(&profile_url).await?;

    wait().await;
    click_collab_simple(&driver).await?;
    wait_in_sec(15).await;

    for item in test.iter() {
        if let Err(e) = process_item_with_delay(15, item, &driver).await {
            eprintln!("Ошибка при клике по '{}': {}", item, e);
        }
    }

    driver.quit().await?;

    Ok(())
}


#[cfg(test)]

mod tests {

    //use crate::add_magyar;
    use super::*;
    #[test]
    fn test_add(){
        assert_eq!(3, add_magyar(1, 2));
    }

    #[tokio::test]
    async fn test_websocket_async() {
        assert!(connect().await.is_ok());
    }
}