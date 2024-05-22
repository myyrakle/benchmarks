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

fn do_iterator_test(iter_test: Vec<i32>) -> Vec<f64> {
    iter_test.iter().map(|&x| x as f64).collect()
}

fn do_for_test(iter_test: Vec<i32>) -> Vec<f64> {
    let mut result = Vec::new();
    result.reserve(iter_test.len());
    for i in 0..iter_test.len() {
        result.push(iter_test[i] as f64);
    }
    result
}

fn main() {
    const N: usize = 1_000_000_000;

    let mut for_test = vec![1; N];
    let mut iter_test = vec![1; N];
    for i in 0..N {
        for_test[i] = i as i32;
        iter_test[i] = i as i32;
    }

    let timer = Timer::new();
    let sum_1 = do_for_test(for_test);
    println!("for: {}ms", timer.elapsed_as_millis());

    let timer = Timer::new();
    let sum_2 = do_iterator_test(iter_test);
    println!("iter: {}ms", timer.elapsed_as_millis());

    assert_eq!(sum_1, sum_2);
}
