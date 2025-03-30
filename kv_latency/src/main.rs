use kv_latency::postgres;

fn main() {
    let mut client = postgres::create_postgres_client().unwrap();
    postgres::init_schema(&mut client).unwrap();

    let key = "asdf";
    let value = "qwerty";
    postgres::set_key_value(&mut client, key, value).unwrap();

    let value = postgres::get_key_value(&mut client, key).unwrap();
    println!("Key: {}, Value: {}", key, value);
}
