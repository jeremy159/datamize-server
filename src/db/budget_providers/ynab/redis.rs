use redis::{Commands, Connection, RedisResult};

pub fn get_categories_delta(redis_conn: &mut Connection) -> Option<i64> {
    redis_conn.get("categories_delta").ok()
}

pub fn set_categories_detla(redis_conn: &mut Connection, server_knowledge: i64) -> RedisResult<()> {
    redis_conn.set("categories_delta", server_knowledge)?;
    Ok(())
}

pub fn del_categories_detla(redis_conn: &mut Connection) -> RedisResult<()> {
    redis_conn.del("categories_delta")
}

pub fn get_categories_last_saved(redis_conn: &mut Connection) -> Option<String> {
    redis_conn.get("categories_last_saved").ok()
}

pub fn set_categories_last_saved(
    redis_conn: &mut Connection,
    last_saved: String,
) -> RedisResult<()> {
    redis_conn.set("categories_last_saved", last_saved)?;
    Ok(())
}

pub fn get_scheduled_transactions_delta(redis_conn: &mut Connection) -> Option<i64> {
    redis_conn.get("scheduled_transactions_delta").ok()
}

pub fn set_scheduled_transactions_delta(
    redis_conn: &mut Connection,
    server_knowledge: i64,
) -> RedisResult<()> {
    redis_conn.set("scheduled_transactions_delta", server_knowledge)?;
    Ok(())
}

pub fn del_scheduled_transactions_delta(redis_conn: &mut Connection) -> RedisResult<()> {
    redis_conn.del("scheduled_transactions_delta")
}

pub fn get_scheduled_transactions_last_saved(redis_conn: &mut Connection) -> Option<String> {
    redis_conn.get("scheduled_transactions_last_saved").ok()
}

pub fn set_scheduled_transactions_last_saved(
    redis_conn: &mut Connection,
    last_saved: String,
) -> RedisResult<()> {
    redis_conn.set("scheduled_transactions_last_saved", last_saved)?;
    Ok(())
}

pub fn get_accounts_delta(redis_conn: &mut Connection) -> Option<i64> {
    redis_conn.get("accounts_delta").ok()
}

pub fn set_accounts_detla(redis_conn: &mut Connection, server_knowledge: i64) -> RedisResult<()> {
    redis_conn.set("accounts_delta", server_knowledge)?;
    Ok(())
}

pub fn get_payees_delta(redis_conn: &mut Connection) -> Option<i64> {
    redis_conn.get("payees_delta").ok()
}

pub fn set_payees_detla(redis_conn: &mut Connection, server_knowledge: i64) -> RedisResult<()> {
    redis_conn.set("payees_delta", server_knowledge)?;
    Ok(())
}
