rs-thread_pool [![Build Status](https://travis-ci.org/nathanfaucett/rs-thread_pool.svg?branch=master)](https://travis-ci.org/nathanfaucett/rs-thread_pool)
=====

thread pool

```toml
[dependencies]
thread_pool = {git = "https://github.com/nathanfaucett/rs-thread_pool"}
```

```rust
extern crate thread_pool


use std::sync::mpsc;

use thread_pool::ThreadPool;


fn main() {
    let thread_pool = ThreadPool::new();
    let (sender, receiver) = mpsc::channel();

    for i in 0..1024 {
        let sender = sender.clone();

        let _ = thread_pool.run(move || {
            let _ = sender.send(i * i);
        });
    }

    for _ in 0..1024 {
        println!("{:?}", receiver.recv().unwrap());
    }
}
```
