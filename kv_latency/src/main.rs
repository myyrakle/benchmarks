use kv_latency::memcached;

fn main() {
    let mut client = memcached::create_memcached_client().unwrap();
    let key = "asdf";
    let value = "qwerty";
    memcached::set_key_value(&mut client, key, value).unwrap();

    let retrieved_value = memcached::get_key_value(&mut client, key).unwrap();
    println!("Retrieved value: {}", retrieved_value);
}
