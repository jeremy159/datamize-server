use std::fmt;

use anyhow::Context;
use clap::{Args, Parser, Subcommand, ValueEnum};
use datamize_server::db::budget_providers::external::{
    PostgresExternalAccountRepo, RedisEncryptionKeyRepo,
};
use datamize_server::models::budget_providers::{EncryptedPassword, WebScrapingAccount};
use datamize_server::services::budget_providers::{
    ExternalAccountService, ExternalAccountServiceExt,
};
use datamize_server::{get_redis_connection_manager, secrecy::Secret, sqlx_error::Error};
use orion::aead;
use orion::kex::SecretKey;
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

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum AccountType {
    Tfsa, // = CELI
    Rrsp, // = REER
    Rpp,  // = RPA
    Resp, // REEE
    #[default]
    OtherAsset,
    OtherLiability,
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

    let configuration = datamize_server::config::Settings::build()?;
    let db_conn_pool = datamize_server::get_connection_pool(&configuration.database);
    let redis_conn = get_redis_connection_manager(&configuration.redis)
        .await
        .context("failed to get redis connection manager")?;

    let mut external_account_service = ExternalAccountService {
        external_account_repo: Box::new(PostgresExternalAccountRepo { db_conn_pool }),
        encryption_key_repo: Box::new(RedisEncryptionKeyRepo { redis_conn }),
    };

    match args.command {
        Commands::Create(create_args) => {
            create_account(&mut external_account_service, args.name, create_args).await?
        }
        Commands::Update(updated_args) => {
            update_account(&mut external_account_service, args.name, updated_args).await?
        }
    }

    Ok(())
}

async fn create_account(
    external_account_service: &mut impl ExternalAccountServiceExt,
    name: String,
    args: CreateArgs,
) -> anyhow::Result<()> {
    let encryption_key = get_encryption_key(external_account_service).await?;
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

    external_account_service
        .create_external_account(&account)
        .await?;
    println!("Successfully created {:?}", account.name);

    Ok(())
}

async fn update_account(
    external_account_service: &mut impl ExternalAccountServiceExt,
    name: String,
    args: UpdateArgs,
) -> anyhow::Result<()> {
    // check if  account exists
    let Ok(mut account) = external_account_service
        .get_external_account_by_name(&name)
        .await
    else {
        return Err::<(), anyhow::Error>(Error::RowNotFound.into())
            .with_context(|| format!("Account {} does not exist", name));
    };

    if let Some(username) = args.username {
        account.username = username;
    }
    if let Some(password) = args.password {
        let encryption_key = get_encryption_key(external_account_service).await?;
        let encrypted_password = Secret::new(EncryptedPassword::new(aead::seal(
            &encryption_key,
            password.as_bytes(),
        )?));
        account.encrypted_password = encrypted_password;
    }
    if let Some(account_type) = args.account_type {
        account.account_type = account_type.to_string().parse().unwrap();
    }
    if let Some(balance) = args.balance {
        account.balance = (balance * 1000_f32) as i64;
    }

    external_account_service
        .update_external_account(&account)
        .await?;
    println!("Successfully updated {:?}", account.name);

    Ok(())
}

async fn get_encryption_key(
    external_account_service: &mut impl ExternalAccountServiceExt,
) -> anyhow::Result<SecretKey> {
    Ok(match external_account_service.get_encryption_key().await {
        Ok(ref val) => {
            if !val.is_empty() {
                SecretKey::from_slice(val).unwrap()
            } else {
                let key = SecretKey::default();
                external_account_service
                    .set_encryption_key(key.unprotected_as_bytes())
                    .await?;
                key
            }
        }
        Err(_) => {
            let key = SecretKey::default();
            external_account_service
                .set_encryption_key(key.unprotected_as_bytes())
                .await?;
            key
        }
    })
}
