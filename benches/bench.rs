#![feature(test)]


extern crate test;

extern crate num_cpus;
extern crate waiter;

extern crate thread_pool;


use test::Bencher;

use thread_pool::ThreadPool;
use waiter::Waiter;


#[bench]
fn bench_thread_pool(b: &mut Bencher) {
    let cpus = num_cpus::get();
    let thread_pool = ThreadPool::new_with_thread_count(cpus);

    b.iter(|| {
        let waiter = Waiter::new_with_count(cpus);

        for _ in 0..cpus {
            let waiter = waiter.clone();

            let _ = thread_pool.run(move || {
                (0..65536).fold(0, |old, new| old ^ new);
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
            (0..65536).fold(0, |old, new| old ^ new);
        }
    });
}
