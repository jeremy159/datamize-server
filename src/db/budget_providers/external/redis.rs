use redis::{Commands, Connection, RedisResult};

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
