use kv_latency::mysql;

fn main() {
    let mut client = mysql::create_postgres_client().unwrap();
    mysql::init_schema(&mut client).unwrap();

    let key = "asdf";
    let value = "qwerty";
    mysql::set_key_value(&mut client, key, value).unwrap();

    let value = mysql::get_key_value(&mut client, key).unwrap();
    println!("Key: {}, Value: {}", key, value);
}
