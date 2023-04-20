use std::time::Duration;

use fantoccini::{ClientBuilder, Locator};
use orion::{aead, kex::SecretKey};
use secrecy::ExposeSecret;
use tokio::time::sleep;

use crate::web_scraper::parsing::parse_balance;

use super::account::WebScrapingAccount;

// FIXME: Find a way for CSS selectors to locate elements inside custom web components...
pub async fn get_rpp_canada_life_sandryne(
    account: WebScrapingAccount,
    encryption_key: &SecretKey,
    webdriver_location: &str,
) -> anyhow::Result<WebScrapingAccount> {
    let mut account = account;
    let c = ClientBuilder::rustls()
        .connect(webdriver_location)
        .await
        .expect("failed to connect to WebDriver");

    c.goto("https://my.canadalife.com/acceder").await?;

    let password = String::from_utf8(aead::open(
        encryption_key,
        account.encrypted_password.expose_secret().as_ref(),
    )?)?;

    let f = c.form(Locator::Css(".card.login__card")).await?;
    f.set_by_name(
        "climsMyLogin:j_id453:j_id454:j_id455:j_id456:loginForm:username",
        &account.username,
    )
    .await?
    .set_by_name(
        "climsMyLogin:j_id453:j_id454:j_id455:j_id456:loginForm:password",
        &password,
    )
    .await?
    .submit()
    .await?;

    sleep(Duration::from_millis(5000)).await;
    c.goto("https://my.canadalife.com/climsgrsqa?GRSDeepLink=/idp/login?app=0sp0A000000002b&RelayState=%2Fmembers%2Fdashboard%2Fplans%2FPRPP")
        .await?;

    let e = c
        .wait()
        .for_element(Locator::Css(
            ".tile-content .row > div:not(.overview) .balance :last-child",
        ))
        .await?;

    let amt = e.text().await?;
    let amt = amt.replace('\u{202f}', "").replace(',', "."); // TODO: Maybe do this with nom?
    let balance = parse_balance(&amt)?;

    c.close().await?;

    account.balance = balance;

    Ok(account)
}
