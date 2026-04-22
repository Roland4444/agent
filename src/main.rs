use std::fs;
//          python3 -m http.server 9999

use anyhow::{Result, Context};
use thirtyfour::prelude::*;
use tokio::time::{Duration, sleep};
use chrono::Local;
use std::path::Path;


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

const PROFILE_PATH: &str = "PROFILE"; 



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

fn path_profile() -> String {
    read_from_file(PROFILE_PATH).expect("alarm")
}


async fn init_chrome_driver() -> Result<WebDriver> {
    let mut caps = DesiredCapabilities::chrome();

   // let profile_path = r"C:\Users\user\Documents\rust\agent\chrome_profile";

    println!("USED DIRECTORY:: {}", path_profile());

    //let profile_path = r"./chrome_profile";
    std::fs::create_dir_all(path_profile())?;
    let user_data_arg = format!("--user-data-dir={}", path_profile());
    caps.add_arg(&user_data_arg)?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--window-size=1920,1080")?;
    caps.add_arg("--headless=new")?; // при необходимости
    let driver = WebDriver::new("http://localhost:21000", caps).await?;
    Ok(driver)
}


// async fn init_chrome_driver() -> Result<WebDriver> {
//     let mut caps = DesiredCapabilities::chrome();
//     caps.add_arg("--no-sandbox")?;
//     caps.add_arg("--user-data-dir=C:\\Users\\user\\Documents\\rust\\agent\\User Data")?;
//     caps.add_arg("--profile-directory=Default")?;
// //    caps.add_arg("--headless=new")?;  // вместо --headless
//     caps.add_arg("--disable-blink-features=AutomationControlled")?;
//     caps.add_arg("--disable-features=IsolateOrigins,site-per-process")?;
//  //   caps.add_arg("--headless")?;                                //
//     caps.add_arg("--window-size=1920,1080")?;
//     caps.add_arg("--disable-gpu")?;                             //
//     caps.add_arg("--disable-software-rasterizer")?;             //    
//     caps.add_arg("--disable-dev-shm-usage")?;                   //
//   //  caps.add_arg("--remote-debugging-port=9222")?;              //
//     let driver = WebDriver::new("http://localhost:21000", caps).await?;
//     Ok(driver)
// }


async fn process_item_with_delay(_delay: u64, text_collab: &str, driver: &WebDriver) -> Result<()> {
    click_collab_simple_text(&driver, text_collab).await?;
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

async fn login_cad(driver: &WebDriver, username: &str, pass: &str) -> Result<()> {
    // Поле логина
    let login_field = driver
        .query(By::Css("input.b24net-text-input__field[type='text']"))
        .wait(Duration::from_secs(10), Duration::from_millis(500))
        .and_clickable()
        .first()
        .await
        .context("Поле логина не появилось")?;
    login_field.send_keys(username).await?;

    // Кнопка "Продолжить" (первая)
    let continue_btn = driver
        .query(By::Css(".b24net-login-enter-form__continue-btn"))
        .wait(Duration::from_secs(10), Duration::from_millis(500))
        .and_clickable()
        .first()
        .await
        .context("Кнопка 'Продолжить' не появилась")?;
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

    // login_cad(&driver, 
    //     login().expect("SHIT HAPPENS").as_str(), 
    //     pass().expect("SHIT HAPPENS").as_str())
    //     .await?;
    

    take_screenshot(&driver, "after_goto").await?; 


    wait().await;

    take_screenshot(&driver, "before_click_collab").await?; // <-- добавлен


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




