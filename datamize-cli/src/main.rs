use std::fmt;

use anyhow::Context;
use clap::{Args, Parser, Subcommand, ValueEnum};
use datamize::web_scraper::account::{EncryptedPassword, WebScrapingAccount};
use datamize::{
    db::budget_providers::external::{
        add_new_external_account, get_external_account_by_name, set_encryption_key,
        update_external_account,
    },
    get_redis_conn, get_redis_connection_pool,
};
use orion::aead;
use orion::kex::SecretKey;
use redis::Connection;
use secrecy::Secret;
use sqlx::PgPool;
use uuid::Uuid;

/// Simple program to quickly perform some operations
/// on some Datamize functionnality without a GUI.
/// In this case, it can be used to create or update some
/// web scrapping accounts.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Name of the account
    name: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new external account
    Create(CreateArgs),
    /// Update an existing external account
    Update(UpdateArgs),
}

#[derive(Args, Debug)]
struct CreateArgs {
    /// The username to connect to the account
    #[arg(short, long)]
    username: String,

    /// The password to connect to the account. Will be encrypted as soon as received.
    #[arg(short, long)]
    password: String,

    /// The type the account will have
    #[arg(short, long, default_value_t)]
    account_type: AccountType,

    /// A starting balance to use for the account
    #[arg(short, long, default_value_t = 0.0)]
    balance: f32,
}

#[derive(Args, Debug)]
struct UpdateArgs {
    /// The username to connect to the account
    #[arg(short, long)]
    username: Option<String>,

    /// The password to connect to the account. Will be encrypted as soon as received.
    #[arg(short, long)]
    password: Option<String>,

    /// The type the account should now have
    #[arg(short, long)]
    account_type: Option<AccountType>,

    /// A new balance to use for the account
    #[arg(short, long)]
    balance: Option<f32>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum AccountType {
    Tfsa, // = CELI
    Rrsp, // = REER
    Rpp,  // = RPA
    Resp, // REEE
    OtherAsset,
    OtherLiability,
}

impl Default for AccountType {
    fn default() -> Self {
        AccountType::OtherAsset
    }
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AccountType::Tfsa => write!(f, "tfsa"),
            AccountType::Rrsp => write!(f, "rrsp"),
            AccountType::Rpp => write!(f, "rpp"),
            AccountType::Resp => write!(f, "resp"),
            AccountType::OtherAsset => write!(f, "other-asset"),
            AccountType::OtherLiability => write!(f, "other-liability"),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let configuration = datamize::config::Settings::build()?;
    let db_conn_pool = datamize::get_connection_pool(&configuration.database);
    let redis_conn_pool = get_redis_connection_pool(&configuration.redis)
        .context("failed to get redis connection pool")?;
    let mut redis_conn =
        get_redis_conn(&redis_conn_pool).context("failed to get redis connection from pool")?;

    match args.command {
        Commands::Create(create_args) => {
            create_account(&mut redis_conn, &db_conn_pool, args.name, create_args).await?
        }
        Commands::Update(updated_args) => {
            update_account(&mut redis_conn, &db_conn_pool, args.name, updated_args).await?
        }
    }

    Ok(())
}

async fn create_account(
    redis_conn: &mut Connection,
    db_conn_pool: &PgPool,
    name: String,
    args: CreateArgs,
) -> anyhow::Result<()> {
    let encryption_key = get_encryption_key(redis_conn)?;
    let encrypted_password = Secret::new(EncryptedPassword::new(aead::seal(
        &encryption_key,
        args.password.as_bytes(),
    )?));

    let account = WebScrapingAccount {
        id: Uuid::new_v4(),
        name,
        account_type: args.account_type.to_string().parse().unwrap(),
        balance: (args.balance * 1000_f32) as i64,
        username: args.username,
        encrypted_password,
        deleted: false,
    };

    add_new_external_account(db_conn_pool, &account).await?;
    println!("Successfully created {:?}", account.name);

    Ok(())
}

async fn update_account(
    redis_conn: &mut Connection,
    db_conn_pool: &PgPool,
    name: String,
    args: UpdateArgs,
) -> anyhow::Result<()> {
    // check if  account exists
    let account = get_external_account_by_name(db_conn_pool, &name).await?;
    if account.is_none() {
        return Err::<(), anyhow::Error>(sqlx::Error::RowNotFound.into())
            .with_context(|| format!("Account {} does not exist", name));
    }

    let mut new_account = account.unwrap();
    if let Some(username) = args.username {
        new_account.username = username;
    }
    if let Some(password) = args.password {
        let encryption_key = get_encryption_key(redis_conn)?;
        let encrypted_password = Secret::new(EncryptedPassword::new(aead::seal(
            &encryption_key,
            password.as_bytes(),
        )?));
        new_account.encrypted_password = encrypted_password;
    }
    if let Some(account_type) = args.account_type {
        new_account.account_type = account_type.to_string().parse().unwrap();
    }
    if let Some(balance) = args.balance {
        new_account.balance = (balance * 1000_f32) as i64;
    }

    update_external_account(db_conn_pool, &new_account).await?;
    println!("Successfully updated {:?}", new_account.name);

    Ok(())
}

fn get_encryption_key(redis_conn: &mut Connection) -> anyhow::Result<SecretKey> {
    Ok(
        match datamize::db::budget_providers::external::get_encryption_key(redis_conn) {
            Some(ref val) => {
                if !val.is_empty() {
                    SecretKey::from_slice(val).unwrap()
                } else {
                    let key = SecretKey::default();
                    set_encryption_key(redis_conn, key.unprotected_as_bytes())?;
                    key
                }
            }
            None => {
                let key = SecretKey::default();
                set_encryption_key(redis_conn, key.unprotected_as_bytes())?;
                key
            }
        },
    )
}
