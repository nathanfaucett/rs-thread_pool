#![feature(test)]


extern crate test;

extern crate thread_pool;


use std::sync::mpsc;
use std::usize;

use test::Bencher;

use thread_pool::ThreadPool;


const SIZE: usize = 1024usize;

#[cfg(target_pointer_width = "64")]
const FAC: usize = 20usize;

#[cfg(target_pointer_width = "32")]
const FAC: usize = 12usize;


fn fac(x: usize) -> usize {
    if x == 0usize {
        1usize
    } else {
        x * fac(x - 1usize)
    }
}

#[bench]
fn bench_threads(b: &mut Bencher) {
    let thread_pool = ThreadPool::new();

    b.iter(|| {
        let (sender, receiver) = mpsc::channel();

        for _ in 0..SIZE {
            let sender = sender.clone();

            let _ = thread_pool.run(move || {
                let mut out = 0usize;
                for _ in 0..SIZE {
                    out = out.wrapping_add(fac(FAC));
                }
                let _ = sender.send(out);
            });
        }

        for _ in 0..SIZE {
            assert_ne!(receiver.recv().unwrap(), 0usize);
        }
    });
}

#[bench]
fn bench_single_thread(b: &mut Bencher) {
    b.iter(|| {
        let mut values = Vec::with_capacity(SIZE);

        for _ in 0..SIZE {
             values.push({
                 let mut out = 0usize;
                 for _ in 0..SIZE {
                     out = out.wrapping_add(fac(FAC));
                 }
                 out
             });
        }

        for value in values.iter() {
            assert_ne!(*value, 0usize);
        }
    });
}
