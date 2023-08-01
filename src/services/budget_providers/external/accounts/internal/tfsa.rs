use fantoccini::{ClientBuilder, Locator};
use orion::aead;
use orion::kex::SecretKey;
use secrecy::ExposeSecret;

use crate::models::budget_providers::WebScrapingAccount;

use super::parsing::parse_balance;

pub async fn get_tfsa(
    account: WebScrapingAccount,
    encryption_key: &SecretKey,
    webdriver_location: &str,
) -> anyhow::Result<WebScrapingAccount> {
    let mut account = account;
    let c = ClientBuilder::rustls()
        .connect(webdriver_location)
        .await
        .expect("failed to connect to WebDriver");

    c.goto("https://www.monpeakenligne.com/secure_new/default.asp?Lng=FR")
        .await?;

    let password = String::from_utf8(aead::open(
        encryption_key,
        account.encrypted_password.expose_secret().as_ref(),
    )?)?;

    let f = c.form(Locator::Css("#login")).await?;
    f.set_by_name("signInEmail", &account.username)
        .await?
        .set_by_name("signInPassword", &password)
        .await?
        .submit_with(Locator::Css("#login .d-flex a"))
        .await?;

    let e = c
        .find(Locator::Css("#dash-all .row .d-flex h2.heading-large"))
        .await?;
    let amt = e.text().await?;
    let balance = parse_balance(&amt)?;

    c.close().await?;

    account.balance = balance;

    Ok(account)
}
