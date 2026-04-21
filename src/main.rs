use std::fs;

use anyhow::{Result, Context};
use thirtyfour::prelude::*;
use tokio::time::{Duration, sleep};
//use futures_util::{SinkExt, StreamExt};
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

const PASS_FIELNAME: &str = "pass";
const LOGIN_FILENAME: &str = "login";



fn pass() -> Option<String> {
    read_from_file(PASS_FIELNAME)
}

fn login() -> Option<String> {
    read_from_file(LOGIN_FILENAME)
}

fn read_from_file(filename: &str) -> Option<String> {
    let g  = fs::read_to_string(filename);
    match g {
        Ok(str) => Some(str),
        Err(_ ) => None,
    }
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

async fn init_chrome_driver() -> Result<WebDriver> {
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--headless")?;                                //
    caps.add_arg("--window-size=1920,1080")?;
    caps.add_arg("--disable-gpu")?;                             //
    caps.add_arg("--disable-software-rasterizer")?;             //    
    caps.add_arg("--disable-dev-shm-usage")?;                   //
  //  caps.add_arg("--remote-debugging-port=9222")?;              //
    let driver = WebDriver::new("http://localhost:21000", caps).await?;
    Ok(driver)
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


async fn login_cad(driver: &WebDriver, username: &str, pass: &str) -> Result<()>{
    let login_field = driver
        .query(By::Css("input.b24net-text-input__field[type='text']"))
        .wait(Duration::from_secs(5), Duration::from_millis(500))
        .first()
        .await;
    

    if let Ok(field) = login_field {
        println!("ENTERING...");
        field.send_keys(username).await?;

        field.send_keys(Key::Enter).await?;

        sleep(Duration::from_secs(15)).await;



        let password_field = driver
            .query(By::Css("input.b24net-text-input__field[type='password']"))
            .wait(Duration::from_secs(2), Duration::from_millis(500))
            .first()
            .await?;

        password_field.send_keys(pass).await?;
        password_field.send_keys(Key::Enter).await?;
    }
    Ok(())
}

const BASE_URL: &str = "https://relits.bitrix24.ru";


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
        OWN
    ];
    const BASE_URL: &str = "https://relits.bitrix24.ru";
    let user_id = 1;
    let driver = init_chrome_driver().await?;

    let profile_url = format!("{}/company/personal/user/{}", BASE_URL, user_id);
    println!("LINK::{}", profile_url);
    driver.goto(&profile_url).await?;

    login_cad(&driver, 
        login().expect("SHIT HAPPENS").as_str(), 
        pass().expect("SHIT HAPPENS").as_str())
        .await?;
    
    wait().await;
    click_collab_simple(&driver).await?;
    wait_in_sec(15).await;

    for item in iterate.iter() {
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


    #[test]
    fn test_read_login(){
        let login_file = "login.txt";
        let login = "rpastushkovb@relits.ru";
        let _ =  fs::write(login_file, login);
        let readed = read_from_file(login_file).expect("not found");
        println!("READED:: {}", readed);
        assert_eq!(login.to_string(), read_from_file(login_file).expect("PANIC"));
    }
}