use redis::{Commands, Connection, RedisResult};

pub fn get_categories_delta(redis_conn: &mut Connection) -> Option<i64> {
    redis_conn.get("categories_delta").ok()
}

pub fn set_categories_detla(redis_conn: &mut Connection, server_knowledge: i64) -> RedisResult<()> {
    redis_conn.set("categories_delta", server_knowledge)?;
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

pub fn get_accounts_delta(redis_conn: &mut Connection) -> Option<i64> {
    redis_conn.get("accounts_delta").ok()
}

pub fn set_accounts_detla(redis_conn: &mut Connection, server_knowledge: i64) -> RedisResult<()> {
    redis_conn.set("accounts_delta", server_knowledge)?;
    Ok(())
}

pub fn get_encryption_key(redis_conn: &mut Connection) -> Option<Vec<u8>> {
    redis_conn.get("encryption_key").ok()
}

pub fn set_encryption_key(
    redis_conn: &mut Connection,
    encryption_key_str: &[u8],
) -> RedisResult<()> {
    redis_conn.set("encryption_key", encryption_key_str)?;
    Ok(())
}
