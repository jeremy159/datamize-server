pub mod account;
pub mod parsing;
pub mod rpp;
pub mod rrsp;
pub mod tfsa;

use axum::{extract::State, Json};
use futures::{future::BoxFuture, stream::FuturesOrdered, StreamExt};
use orion::kex::SecretKey;
use redis::Connection;
use sqlx::PgPool;

use crate::{config, db, error::HttpJsonAppResult, startup::AppState};

use self::account::{AccountType, WebScrapingAccount};

#[tracing::instrument(name = "Get web scraper testing route", skip_all)]
pub async fn get_web_scraper(State(_app_state): State<AppState>) -> HttpJsonAppResult<()> {
    // let db_conn_pool = app_state.db_conn_pool;
    // let mut redis_conn = get_redis_conn(&app_state.redis_conn_pool)
    //     .context("failed to get redis connection from pool")?;

    // let accounts = get_external_accounts(&db_conn_pool, &mut redis_conn).await?;

    // for a in &accounts {
    //     println!("{:?}: {:?}", a.name, a.balance);
    // }

    let res = rrsp::get_rrsp_jeremy().await?;
    println!("{:?}", &res);

    Ok(Json(()))
}

#[tracing::instrument(skip_all)]
pub async fn get_external_accounts(
    db_conn_pool: &PgPool,
    redis_conn: &mut Connection,
) -> anyhow::Result<Vec<WebScrapingAccount>> {
    let configuration = config::Settings::build()?;
    let webdriver_location = configuration.webdriver.connection_string();

    let encryption_key = match db::get_encryption_key(redis_conn) {
        Some(ref val) => SecretKey::from_slice(val).unwrap(),
        None => {
            let key = SecretKey::default();
            db::set_encryption_key(redis_conn, key.unprotected_as_bytes())?;
            key
        }
    };

    let initial_accounts = db::get_all_external_accounts(db_conn_pool).await?;
    let updated_accounts = initial_accounts
        .clone()
        .into_iter()
        .map(|account| {
            let r: BoxFuture<_> = match account.account_type {
                AccountType::Tfsa => Box::pin(tfsa::get_tfsa(
                    account,
                    &encryption_key,
                    &webdriver_location,
                )),
                AccountType::Rpp => Box::pin(rpp::get_rpp_canada_life_sandryne(
                    account,
                    &encryption_key,
                    &webdriver_location,
                )),
                AccountType::Rrsp => Box::pin(rrsp::get_rrsp_ia_sandryne(
                    account,
                    &encryption_key,
                    &webdriver_location,
                )),
                _ => Box::pin(async { Ok(account) }),
            };
            r
        })
        .collect::<FuturesOrdered<BoxFuture<_>>>()
        .collect::<Vec<_>>()
        .await;

    Ok(updated_accounts
        .into_iter()
        .zip(initial_accounts)
        .map(
            |(updated_account_res, i_account)| match updated_account_res {
                Ok(u_account) => {
                    if u_account.balance != i_account.balance {
                        u_account
                    } else {
                        i_account
                    }
                }
                Err(e) => {
                    tracing::error!(
                        error.cause_chain = ?e,
                        error.message = %e,
                        "Failed to get latest balance for account {}. Skipping.",
                        i_account.name
                    );
                    i_account
                }
            },
        )
        .collect::<Vec<_>>())
}
