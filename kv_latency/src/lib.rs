#![feature(test)]
extern crate test;

pub mod redis;

pub const KV_COUNT: usize = 1000;

pub fn generate_kv_entries(n: usize) -> Vec<(String, String)> {
    let mut entries = Vec::with_capacity(n);
    for i in 0..n {
        entries.push((format!("key{}", i), format!("value{}", i)));
    }

    entries
}

#[bench]
fn bench_redis_set_single(bencher: &mut test::Bencher) {
    let mut connection = redis::create_redis_client().unwrap();

    let key = "asdf";
    let value = "qwerty";

    bencher.iter(|| {
        redis::set_key_value(&mut connection, key, value).unwrap();
    });
}

#[bench]
fn bench_redis_get_single(bencher: &mut test::Bencher) {
    let mut connection = redis::create_redis_client().unwrap();
    let key = "asdf";
    let value = "qwerty";

    redis::set_key_value(&mut connection, key, value).unwrap();

    bencher.iter(|| {
        redis::get_key_value(&mut connection, key).unwrap();
    });
}

#[bench]
fn bench_redis_set_bulk(bencher: &mut test::Bencher) {
    let mut connection = redis::create_redis_client().unwrap();
    let entries = generate_kv_entries(KV_COUNT);

    bencher.iter(|| {
        entries.iter().for_each(|(key, value)| {
            redis::set_key_value(&mut connection, key, value).unwrap();
        });
    });
}

#[bench]
fn bench_redis_get_bulk(bencher: &mut test::Bencher) {
    let mut connection = redis::create_redis_client().unwrap();
    let entries = generate_kv_entries(KV_COUNT);

    bencher.iter(|| {
        entries.iter().for_each(|(key, _)| {
            redis::get_key_value(&mut connection, key).unwrap();
        });
    });
}
