use memcache::Client;

pub fn create_memcached_client() -> anyhow::Result<Client> {
    // connect to redis
    let client = memcache::connect("memcache://0.0.0.0:21211?timeout=10&tcp_nodelay=true").unwrap();

    let _ = client.version()?;

    Ok(client)
}

pub fn set_key_value(connection: &mut Client, key: &str, value: &str) -> anyhow::Result<()> {
    // set key value
    connection.set(key, value, 0)?;

    Ok(())
}

pub fn get_key_value(connection: &mut Client, key: &str) -> anyhow::Result<String> {
    // get key value
    let value = connection.get(key)?;

    Ok(value.unwrap_or_default())
}
