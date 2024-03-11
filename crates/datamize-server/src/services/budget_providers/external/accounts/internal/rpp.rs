use std::time::Duration;

use datamize_domain::{secrecy::ExposeSecret, WebScrapingAccount};
use fantoccini::{ClientBuilder, Locator};
use orion::{aead, kex::SecretKey};
use tokio::time::sleep;

use super::parsing::parse_balance;

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

    c.find(Locator::Id("onetrust-accept-btn-handler"))
        .await?
        .click()
        .await?;

    let password = String::from_utf8(aead::open(
        encryption_key,
        account.encrypted_password.expose_secret().as_ref(),
    )?)?;

    let f = c.form(Locator::Css(".card.login__card")).await?;
    f.set(
        Locator::Css("[name*=\"loginForm:username\"]"),
        &account.username,
    )
    .await?
    .set(Locator::Css("[name*=\"loginForm:password\"]"), &password)
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
    let amt = amt.replace('\u{202f}', "").replace(',', ".");
    let balance = parse_balance(&amt)?;

    c.close().await?;

    account.balance = balance;

    Ok(account)
}
