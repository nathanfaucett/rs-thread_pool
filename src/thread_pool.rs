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
    #[inline(always)]
    fn call_box_fn(self: Box<F>) {
        (*self)()
    }
}


pub type Thunk = Box<BoxFn + Send + 'static>;


struct Thread<'a> {
    active: bool,
    tasks_receiver: &'a Arc<Mutex<Receiver<Thunk>>>,
}

impl<'a> Thread<'a> {
    #[inline(always)]
    fn new(
        tasks_receiver:  &'a Arc<Mutex<Receiver<Thunk>>>,
    ) -> Self {
        Thread {
            active: true,
            tasks_receiver: tasks_receiver,
        }
    }

    #[inline(always)]
    fn is_active(&self) -> bool {
        self.active
    }

    #[inline(always)]
    fn kill(&mut self) {
        self.active = false;
    }
}

impl<'a> Drop for Thread<'a> {
    #[inline(always)]
    fn drop(&mut self) {
        if self.is_active() {
            spawn_thread(self.tasks_receiver.clone());
        }
    }
}


pub struct ThreadPool {
    thread_count: usize,
    tasks: Sender<Thunk>,
}

impl ThreadPool {
    #[inline(always)]
    pub fn new() -> Self {
        Self::from_count(num_cpus::get() - 1)
    }
    pub fn from_count(thread_count: usize) -> Self {
        let (sender, receiver) = channel::<Thunk>();
        let tasks_receiver = Arc::new(Mutex::new(receiver));

        for _ in 0..thread_count {
            spawn_thread(tasks_receiver.clone());
        }

        ThreadPool {
            thread_count: thread_count,
            tasks: sender,
        }
    }
    #[inline(always)]
    pub fn thread_count(&self) -> usize { self.thread_count }
    #[inline(always)]
    pub fn run<F>(&self, func: F) -> Result<(), SendError<Thunk>>
        where F: FnOnce(),
              F: Send + 'static,
    {
        self.tasks.send(Box::new(func))
    }
}

#[inline(always)]
fn spawn_thread(
    tasks_receiver: Arc<Mutex<Receiver<Thunk>>>,
) {
    thread::spawn(move || {
        let mut t = Thread::new(&tasks_receiver);

        loop {
            let msg = tasks_receiver.lock().unwrap().recv();
            
            match msg {
                Ok(func) => func.call_box_fn(),
                Err(..) => break,
            }
        }

        t.kill();
    });
}
