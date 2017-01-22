use std::any::Any;
use std::marker::PhantomData;
use std::sync::mpsc::{channel, Receiver, Sender, SendError};
use std::thread;
use std::sync::{Arc, Mutex};

use num_cpus;


trait FnBox {
    fn call_box(self: Box<Self>) -> BoxAny;
}

impl<F> FnBox for F
    where F: FnOnce() -> BoxAny,
          F: Send + 'static,
{
    fn call_box(self: Box<F>) -> BoxAny {
        (*self)()
    }
}


type BoxAny = Box<Any + Send + 'static>;


struct Data {
    sender: Sender<BoxAny>,
    task: Box<FnBox + Send + 'static>,
}

impl Data {
    fn call(self) -> Result<(), SendError<BoxAny>> {
        let sender = self.sender;
        let task = self.task;
        sender.send(task.call_box())
    }
}


pub struct Handle<T: 'static> {
    phantom_data: PhantomData<T>,
    receiver: Receiver<BoxAny>,
}

impl<T: 'static> Handle<T> {
    fn new(receiver: Receiver<BoxAny>) -> Self {
        Handle {
            phantom_data: PhantomData,
            receiver: receiver,
        }
    }
    pub fn join(self) -> thread::Result<T> {
        match self.receiver.recv() {
            Ok(raw) => match raw.downcast::<T>() {
                Ok(value) => Ok(*value),
                Err(..) => Err(Box::new("could not downcast value to type.")),
            },
            Err(..) => Err(Box::new("could not get value from thread.")),
        }
    }
}


struct Thread {
    active: bool,
    tasks_receiver: Arc<Mutex<Receiver<BoxAny>>>,
}

impl Thread {
    fn new(
        tasks_receiver: Arc<Mutex<Receiver<BoxAny>>>,
    ) -> Self {
        Thread {
            active: true,
            tasks_receiver: tasks_receiver,
        }
    }

    fn is_active(&self) -> bool { self.active }

    fn cancel(&mut self) {
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
    tasks: Sender<BoxAny>,
}

impl ThreadPool {
    pub fn new() -> Self {
        Self::from_count(num_cpus::get() - 1)
    }
    pub fn from_count(thread_count: usize) -> Self {
        let (sender, receiver) = channel::<BoxAny>();
        let tasks_receiver = Arc::new(Mutex::new(receiver));

        for _ in 0..thread_count {
            spawn_in_pool(tasks_receiver.clone());
        }

        ThreadPool {
            tasks: sender,
        }
    }
    pub fn run<F, T>(&self, func: F) -> Handle<T>
        where F: FnOnce() -> T,
              F: Send + 'static,
              T: Send + 'static
    {
        let (sender, receiver) = channel::<BoxAny>();
        let handle = Handle::<T>::new(receiver);
        let data = Box::new(Data {
            sender: sender,
            task: Box::new(move || Box::new(func()) as BoxAny),
        });

        match self.tasks.send(data) {
            Ok(_) => (),
            Err(..) => (),
        }

        handle
    }
}

fn spawn_in_pool(
    tasks_receiver: Arc<Mutex<Receiver<BoxAny>>>,
) {
    thread::spawn(move || {
        let mut t = Thread::new(
            tasks_receiver.clone()
        );

        loop {
            match tasks_receiver.lock() {
                Ok(tasks) => match tasks.recv() {
                    Ok(raw) => match raw.downcast::<Data>() {
                        Ok(data) => match data.call() {
                            Ok(..) => (),
                            Err(..) => break,
                        },
                        Err(..) => break,
                    },
                    Err(..) => break,
                },
                Err(..) => break,
            };
        }

        t.cancel();
    });
}
