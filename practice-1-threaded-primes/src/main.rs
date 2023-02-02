use std::{
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

fn main() {
    const MAX_THREADS: usize = 8;
    let args = std::env::args().collect::<Vec<String>>();
    let mut start = args
        .get(1)
        .expect("Could not find start")
        .parse::<i32>()
        .expect("Coud not parse start");
    let end = args
        .get(2)
        .expect("Could not find end")
        .parse::<i32>()
        .expect("Could not parse end");

    if start % 2 == 0 {
        start += 1
    }

    let mut result = Vec::with_capacity(((end - start) as usize) / 2);
    let mut threads = Vec::with_capacity(MAX_THREADS);

    let timer = Instant::now();
    for i in 0..MAX_THREADS {
        threads.push(thread::spawn(|| {
            let mut res = Vec::new();
            let mut c = start + (i * 2) as i32;
            while c <= end {
                if is_prime(c) {
                    res.push(c);
                }
                c += MAX_THREADS as i32 * 2;
            }
            res
        }))
    }

    for thread in threads {
        result.extend(thread.join().unwrap());
    }
    let elapsed_time = timer.elapsed();

    result.sort();
    println!("{:?}", result);
    println!("Time taken: {:?}", elapsed_time);
}

#[inline]
fn is_prime(n: i32) -> bool {
    if n == 1 {
        return true;
    }

    let mut i = 2;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 1
    }
    true
}
