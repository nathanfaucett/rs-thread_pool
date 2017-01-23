extern crate thread_pool;


use std::sync::mpsc;

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
    let (sender, receiver) = mpsc::channel();

    for _ in 0..SIZE {
        let sender = sender.clone();

        let _ = thread_pool.run(move || {
            let mut out = 0;
            for _ in 0..SIZE {
                fac(FAC);
                out += 1;
            }
            let _ = sender.send(out);
        });
    }

    for _ in 0..SIZE {
        assert_eq!(SIZE, receiver.recv().unwrap());
    }
}
