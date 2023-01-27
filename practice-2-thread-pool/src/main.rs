mod thread_pool;

use std::{thread, time::Duration};

use crate::thread_pool::Pool;

fn task() {
    println!("\tHello, from task, threadid: {:?}", thread::current().id());
    thread::sleep(Duration::from_secs(2));
}

fn main() {
    let mut workers = Pool::new(2);
    //let mut eventloop = Pool::new(1);

    workers.start();
    //eventloop.start();

    for i in 0..5 {
        thread::sleep(std::time::Duration::from_millis(200));
        println!("Posting task nr: {}", i);
        workers.post(task);
        //eventloop.post(task);
    }
    thread::sleep(std::time::Duration::from_secs(5));

    println!("Starting task in 4 secongs");
    //eventloop.post_timeout(task, Duration::from_secs(4));
    workers.post_timeout(task, Duration::from_micros(3000));

    workers.stop_and_finish();
    //eventloop.stop_and_finish();
}
