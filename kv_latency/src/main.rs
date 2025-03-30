#![feature(test)]
extern crate test;

mod redis;

#[bench]
fn bench_example(b: &mut test::Bencher) {
    b.iter(|| {});
}

fn main() {
    println!("Hello, world!");
}
