#![feature(test)]


extern crate test;

extern crate thread_pool;


use test::Bencher;

use thread_pool::ThreadPool;


const SIZE: usize = 1024;

#[cfg(target_pointer_width = "64")]
const FAC: usize = 20;

#[cfg(target_pointer_width = "32")]
const FAC: usize = 12;


fn fac(x: usize) -> usize {
    if x == 0 {
        1
    } else {
        x * fac(x - 1)
    }
}

#[bench]
fn bench_threads(b: &mut Bencher) {
    let thread_pool = ThreadPool::new();

    b.iter(|| {
        let mut handles = Vec::with_capacity(SIZE);

        for _ in 0..SIZE {
            handles.push(thread_pool.run(move || {
                let mut out = 0;
                for _ in 0..SIZE {
                    fac(FAC);
                    out += 1;
                }
                out
            }));
        }

        for handle in handles {
            assert_eq!(SIZE, handle.join().unwrap());
        }
    });
}

#[bench]
fn bench_single_thread(b: &mut Bencher) {
    b.iter(|| {
        let mut values = Vec::with_capacity(SIZE);

        for _ in 0..SIZE {
             values.push({
                 let mut out = 0;
                 for _ in 0..SIZE {
                     fac(FAC);
                     out += 1;
                 }
                 out
             });
        }

        for value in values {
            assert_eq!(SIZE, value);
        }
    });
}
