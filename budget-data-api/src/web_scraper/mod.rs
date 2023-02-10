use crate::config::types::CeliConfig;
use crate::Result;
use reqwest::header;

pub async fn get_current_amount_of_celi(celi_config: &CeliConfig) -> Result<i64> {
    let mut headers = header::HeaderMap::new();
    let cargo_pkg_name = env!("CARGO_PKG_NAME");
    let cargo_pkg_version = env!("CARGO_PKG_VERSION");
    let user_agent_value = format!("{}/{}", cargo_pkg_name, cargo_pkg_version);
    headers.insert(
        reqwest::header::USER_AGENT,
        header::HeaderValue::from_str(&user_agent_value).unwrap(),
    );

    let http_client = reqwest::Client::builder()
        .default_headers(headers)
        .cookie_store(true)
        .build()?;

    http_client
        .post(&celi_config.login_url)
        .form(&celi_config.params)
        .send()
        .await?;

    let res = http_client
        .get(&celi_config.data_url)
        .send()
        .await?
        .text()
        .await?;

    if let Some(i) = res.find('|') {
        let amount = &res[i + 2..res.len() - 1];
        let amount = (amount.parse::<f64>().unwrap() * 1000_f64) as i64; // Amount in milliunits format
        return Ok(amount);
    }

    // TODO: Return error of not found instead.
    Ok(0)
}
