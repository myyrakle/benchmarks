#![feature(test)]
extern crate test;

pub mod memcached;
pub mod mysql;
pub mod postgres;
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

// Memcached

#[bench]
fn bench_memcached_set_single(bencher: &mut test::Bencher) {
    let mut connection = memcached::create_memcached_client().unwrap();

    let key = "asdf";
    let value = "qwerty";

    bencher.iter(|| {
        memcached::set_key_value(&mut connection, key, value).unwrap();
    });
}

#[bench]
fn bench_memcached_get_single(bencher: &mut test::Bencher) {
    let mut connection = memcached::create_memcached_client().unwrap();
    let key = "asdf";
    let value = "qwerty";

    memcached::set_key_value(&mut connection, key, value).unwrap();

    bencher.iter(|| {
        memcached::get_key_value(&mut connection, key).unwrap();
    });
}

#[bench]
fn bench_memcached_set_bulk(bencher: &mut test::Bencher) {
    let mut connection = memcached::create_memcached_client().unwrap();
    let entries = generate_kv_entries(KV_COUNT);

    bencher.iter(|| {
        entries.iter().for_each(|(key, value)| {
            memcached::set_key_value(&mut connection, key, value).unwrap();
        });
    });
}

#[bench]
fn bench_memcached_get_bulk(bencher: &mut test::Bencher) {
    let mut connection = memcached::create_memcached_client().unwrap();
    let entries = generate_kv_entries(KV_COUNT);

    bencher.iter(|| {
        entries.iter().for_each(|(key, _)| {
            memcached::get_key_value(&mut connection, key).unwrap();
        });
    });
}

// postgres bench
#[bench]
fn bench_postgres_set_single(bencher: &mut test::Bencher) {
    let mut client = postgres::create_postgres_client().unwrap();
    postgres::init_schema(&mut client).unwrap();

    let key = "asdf";
    let value = "qwerty";

    bencher.iter(|| {
        postgres::set_key_value(&mut client, key, value).unwrap();
    });
}

#[bench]
fn bench_postgres_get_single(bencher: &mut test::Bencher) {
    let mut client = postgres::create_postgres_client().unwrap();
    postgres::init_schema(&mut client).unwrap();

    let key = "asdf";
    let value = "qwerty";

    postgres::set_key_value(&mut client, key, value).unwrap();

    bencher.iter(|| {
        postgres::get_key_value(&mut client, key).unwrap();
    });
}

#[bench]
fn bench_postgres_set_bulk(bencher: &mut test::Bencher) {
    let mut client = postgres::create_postgres_client().unwrap();
    postgres::init_schema(&mut client).unwrap();

    let entries = generate_kv_entries(KV_COUNT);

    bencher.iter(|| {
        entries.iter().for_each(|(key, value)| {
            postgres::set_key_value(&mut client, key, value).unwrap();
        });
    });
}

#[bench]
fn bench_postgres_get_bulk(bencher: &mut test::Bencher) {
    let mut client = postgres::create_postgres_client().unwrap();
    postgres::init_schema(&mut client).unwrap();

    let entries = generate_kv_entries(KV_COUNT);

    entries.iter().for_each(|(key, value)| {
        postgres::set_key_value(&mut client, key, value).unwrap();
    });

    bencher.iter(|| {
        entries.iter().for_each(|(key, _)| {
            postgres::get_key_value(&mut client, key).unwrap();
        });
    });
}
