extern crate thread_pool;


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

#[test]
fn test() {
    let thread_pool = ThreadPool::new();

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
}
