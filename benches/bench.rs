#![feature(test)]


extern crate test;

extern crate num_cpus;
extern crate waiter;
extern crate thread_pool;


use std::{thread, time};

use test::Bencher;

use waiter::Waiter;
use thread_pool::ThreadPool;


#[bench]
fn bench_threads(b: &mut Bencher) {
    let cpus = num_cpus::get();
    let thread_pool = ThreadPool::from_count(cpus);

    b.iter(|| {
        let waiter = Waiter::new();

        for _ in 0..cpus {
            let waiter = waiter.clone();

            let _ = thread_pool.run(move || {
                thread::sleep(time::Duration::from_millis(1));
                waiter.done();
            });
        }

        waiter.wait();
    });
}

#[bench]
fn bench_single_thread(b: &mut Bencher) {
    let cpus = num_cpus::get();

    b.iter(|| {
        for _ in 0..cpus {
            thread::sleep(time::Duration::from_millis(1));
        }
    });
}
