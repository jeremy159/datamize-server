use std::time::Duration;

use fantoccini::{ClientBuilder, Locator};
use orion::{aead, kex::SecretKey};
use secrecy::ExposeSecret;
use tokio::time::sleep;

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

pub async fn get_rrsp_jeremy() -> anyhow::Result<String> {
    let c = ClientBuilder::rustls()
        .connect("http://127.0.0.1:4444")
        .await
        .expect("failed to connect to WebDriver");

    c.goto("https://id.manulife.ca/?ui_locales=fr-CA&goto=https%3A%2F%2Fportal.manulife.ca%2Fapps%2Fgroupretirement%2Fportal%2Fmember%2Fhandlelogin%3Fui_locales%3Den-CA")
        .await?;
    let url = c.current_url().await?;
    println!("{:?}", url.as_ref());

    // c.wait()
    //     .for_element(Locator::Css(".Form__StyledForm-sntzww-0"))
    //     .await?;
    let f = c.form(Locator::Css(".Form__StyledForm-sntzww-0")).await?;
    c.find(Locator::Id("username")).await?.send_keys("").await?;
    c.find(Locator::Id("password")).await?.send_keys("").await?;
    f.submit().await?;

    sleep(Duration::from_millis(30000)).await;

    c.wait()
        .for_element(Locator::Id("button-id-lge41jnm"))
        .await?;

    let url = c.current_url().await?;
    println!("{:?}", url.as_ref());
    // TODO: Handle MFA authentification here
    // * Need to click on email button to send MFA request to email
    // * Gettings access to Outlook with https://learn.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-client-creds-grant-flow OR https://github.com/sreeise/graph-rs-sdk
    // * Read emails with https://learn.microsoft.com/en-us/outlook/rest/get-started#calling-the-mail-api
    // * And finally to enter code from email into web page to continue flow
    c.find(Locator::Css("#button-id-lge41jnm"))
        .await?
        .click()
        .await?;

    let e = c
        .find(Locator::Css("#assetbalance .listelementTotalValue"))
        .await?;
    let amt = e.text().await?;
    println!("{:?}", &amt);

    c.close().await?;

    Ok(amt)
}
