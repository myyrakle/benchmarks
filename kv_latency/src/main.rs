use kv_latency::rstore;

fn main() {
    let client = rstore::create_rstore_client().unwrap();

    let key = "asdf";
    let value = "qwerty";

    rstore::set_key_value(&client, key, value).unwrap();

    let retrieved_value = rstore::get_key_value(&client, key).unwrap();
    println!("Retrieved value: {}", retrieved_value);
}
