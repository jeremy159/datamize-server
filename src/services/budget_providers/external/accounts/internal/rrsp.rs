use fantoccini::{ClientBuilder, Locator};
use orion::{aead, kex::SecretKey};
use secrecy::ExposeSecret;

use crate::models::budget_providers::WebScrapingAccount;

use super::parsing::parse_balance;

pub async fn get_rrsp_ia_sandryne(
    account: WebScrapingAccount,
    encryption_key: &SecretKey,
    webdriver_location: &str,
) -> anyhow::Result<WebScrapingAccount> {
    let mut account = account;

    let c = ClientBuilder::rustls()
        .connect(webdriver_location)
        .await
        .expect("failed to connect to WebDriver");

    c.goto("https://clients.ia.ca/account/login?fromURI=https%3A%2F%2Flogin.service.ia.ca%2Fapp%2Fia-ia_extranetsiteminderclients_2%2Fexk1d12zt32HeLOEQ5d7%2Fsso%2Fsaml%3FRelayState%3Df0a051f868d63e5a3a93ca87b07a95cf11a02553")
        .await?;

    let password = String::from_utf8(aead::open(
        encryption_key,
        account.encrypted_password.expose_secret().as_ref(),
    )?)?;

    let f = c.form(Locator::Css("#form19")).await?;
    c.find(Locator::Id("okta-signin-username"))
        .await?
        .send_keys(&account.username)
        .await?;
    c.find(Locator::Id("okta-signin-password"))
        .await?
        .send_keys(&password)
        .await?;

    f.submit().await?;
    c.wait().for_element(Locator::Id("divMyContracts")).await?;
    c.find(Locator::Css("#divMyContracts a"))
        .await?
        .click()
        .await?;

    c.wait()
        .for_element(Locator::Css(
            ".sommaire .RC_dualBlocks .setOfBlocks .bgColor1 p.number",
        ))
        .await?;
    let e = c
        .find(Locator::Css(
            ".sommaire .RC_dualBlocks .setOfBlocks .bgColor1 p.number",
        ))
        .await?;
    let amt = e.text().await?;
    let balance = parse_balance(&amt)?;

    c.close().await?;

    account.balance = balance;

    Ok(account)
}
