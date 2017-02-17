extern crate num_cpus;
extern crate waiter;
extern crate thread_pool;


use std::{thread, time};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use waiter::Waiter;
use thread_pool::ThreadPool;


#[test]
fn test_threads() {
    let cpus = num_cpus::get();
    let thread_pool = ThreadPool::new_with_thread_count(cpus);

    let waiter = Waiter::new_with_count(cpus);
    let counter = Arc::new(AtomicUsize::new(0usize));

    for _ in 0..cpus {
        let waiter = waiter.clone();
        let counter = counter.clone();

        let _ = thread_pool.run(move || {
            thread::sleep(time::Duration::from_millis(1));
            counter.fetch_add(1usize, Ordering::Relaxed);
            waiter.done();
        });
    }

    waiter.wait();
    assert_eq!(counter.load(Ordering::Relaxed), cpus);
}
