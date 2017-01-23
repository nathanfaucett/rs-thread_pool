use std::sync::mpsc::{channel, Receiver, Sender, SendError};
use std::thread;
use std::sync::{Arc, Mutex};

use num_cpus;


pub trait BoxFn {
    fn call_box_fn(self: Box<Self>);
}

impl<F> BoxFn for F
    where F: FnOnce(),
          F: Send + 'static,
{
    fn call_box_fn(self: Box<F>) {
        (*self)()
    }
}


pub type Thunk = Box<BoxFn + Send + 'static>;


struct Thread {
    active: bool,
    tasks_receiver: Arc<Mutex<Receiver<Thunk>>>,
}

impl Thread {
    fn new(
        tasks_receiver: Arc<Mutex<Receiver<Thunk>>>,
    ) -> Self {
        Thread {
            active: true,
            tasks_receiver: tasks_receiver,
        }
    }

    fn is_active(&self) -> bool { self.active }

    fn kill(&mut self) {
        self.active = false;
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        if self.is_active() {
            spawn_in_pool(
                self.tasks_receiver.clone()
            );
        }
    }
}


pub struct ThreadPool {
    tasks: Sender<Thunk>,
}

impl ThreadPool {
    pub fn new() -> Self {
        Self::from_count(num_cpus::get() - 1)
    }
    pub fn from_count(thread_count: usize) -> Self {
        let (sender, receiver) = channel::<Thunk>();
        let tasks_receiver = Arc::new(Mutex::new(receiver));

        for _ in 0..thread_count {
            spawn_in_pool(tasks_receiver.clone());
        }

        ThreadPool {
            tasks: sender,
        }
    }
    pub fn run<F>(&self, func: F) -> Result<(), SendError<Thunk>>
        where F: FnOnce(),
              F: Send + 'static,
    {
        self.tasks.send(Box::new(func))
    }
}

fn spawn_in_pool(
    tasks_receiver: Arc<Mutex<Receiver<Thunk>>>,
) {
    thread::spawn(move || {
        let mut t = Thread::new(
            tasks_receiver.clone()
        );

        loop {
            match tasks_receiver.lock() {
                Ok(tasks) => match tasks.recv() {
                    Ok(func) => func.call_box_fn(),
                    Err(..) => break,
                },
                Err(..) => break,
            };
        }

        t.kill();
    });
}
