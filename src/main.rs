use anyhow::Result;
use thirtyfour::prelude::*;
use tokio::time::{sleep, Duration};

pub async fn click_collab_simple(driver: &WebDriver) -> Result<()> {
    let by_xpath = By::XPath("//*[text()='Коллабы']");
    let element = driver.find(by_xpath).await?;
    element.click().await?;
    Ok(())
}

pub async fn click_collab_simple_text(driver: &WebDriver, text: &str) -> Result<()> {
    let condition = format!("//*[text()='{}']", text);
    let by_xpath = By::XPath(condition);
    let element = driver.find(by_xpath).await?;
    element.click().await?;
    Ok(())
}

async fn wait() -> Result<()> {
    sleep(Duration::from_secs(30)).await;
    Ok(())
}

async fn wait_in_sec(delay: u64) -> Result<()> {
    sleep(Duration::from_secs(delay)).await;
    Ok(())
}

async fn init_chrome_driver() -> Result<WebDriver> {
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    let driver = WebDriver::new("http://localhost:21000", caps).await?;
    Ok(driver)
}

#[tokio::main]
async fn main() -> Result<()> {
    const BASE_URL: &str = "https://relits.bitrix24.ru";
    let user_id = 1;
    let driver = init_chrome_driver().await?;

    let profile_url = format!("{}/company/personal/user/{}", BASE_URL, user_id);
    println!("LINK::{}", profile_url);
    driver.goto(&profile_url).await?;

    wait().await?;              
    click_collab_simple(&driver).await?;
    wait_in_sec(15).await?;   

    click_collab_simple_text(&driver, "Платежи").await?;  ///works!
    wait_in_sec(15).await?;


    driver.quit().await?;

    Ok(())
}