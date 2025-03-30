use redis::{Commands, Connection};

pub fn create_redis_client() -> anyhow::Result<Connection> {
    // connect to redis
    let client = redis::Client::open("redis://0.0.0.0:16379/")?;
    let mut connection = client.get_connection()?;

    let _: () = connection.ping()?;

    Ok(connection)
}

pub fn set_key_value(connection: &mut Connection, key: &str, value: &str) -> anyhow::Result<()> {
    // set key value
    let _: () = connection.set(key, value)?;

    Ok(())
}

pub fn get_key_value(connection: &mut Connection, key: &str) -> anyhow::Result<String> {
    // get key value
    let value: String = connection.get(key)?;

    Ok(value)
}
