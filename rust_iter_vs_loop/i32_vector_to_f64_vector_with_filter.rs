pub struct Timer {
    start: std::time::Instant,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            start: std::time::Instant::now(),
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }

    pub fn elapsed_as_millis(&self) -> u128 {
        self.elapsed().as_millis()
    }

    pub fn elapsed_as_secs(&self) -> u64 {
        self.elapsed().as_secs()
    }
}

fn do_iterator_test(input: Vec<i32>) -> Vec<f64> {
    input
        .iter()
        .map(|&x| x as f64)
        .filter(|x| (*x as i64) % 2 == 0)
        .collect()
}

fn do_loop_test(input: Vec<i32>) -> Vec<f64> {
    let mut result = vec![];
    result.reserve(input.len());
    let mut i = 0;
    while i < input.len() {
        let e = input[i] as f64;

        if (e as i64) % 2 == 0 {
            result.push(e);
        }

        i += 1;
    }
    result
}

fn main() {
    const N: usize = 1_000_000_000;

    let mut loop_test = vec![1; N];
    let mut iter_test = vec![1; N];
    for i in 0..N {
        loop_test[i] = i as i32;
        iter_test[i] = i as i32;
    }

    let timer_1 = Timer::new();
    let sum_1 = do_loop_test(loop_test);
    println!("loop: {}ms", timer_1.elapsed_as_millis());

    let timer_2 = Timer::new();
    let sum_2 = do_iterator_test(iter_test);
    println!("iter: {}ms", timer_2.elapsed_as_millis());

    assert_eq!(sum_1, sum_2);
}
