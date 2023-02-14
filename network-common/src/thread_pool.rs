use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct Pool<F> where
F: FnOnce() + Send + 'static {
    workers: u32,
    tasks: Arc<Mutex<VecDeque<F>>>,
    condvar: Arc<(Mutex<bool>, Condvar)>,
    run_threads: Vec<JoinHandle<()>>,
    running: Arc<Mutex<bool>>,
}

impl<F> Pool<F> where
F: FnOnce() + Send + 'static {
    pub fn new(workers: u32) -> Self {
        Self {
            workers,
            tasks: Arc::new(Mutex::new(VecDeque::new())),
            condvar: Arc::new((Mutex::new(false), Condvar::new())),
            run_threads: Vec::new(),
            running: Arc::new(Mutex::new(true)),
        }
    }

    pub fn start(&mut self) {
        // Checking if start has already been ran
        if !self.run_threads.is_empty() {
            return;
        }

        // Creating all the threads
        for _ in 0..self.workers {
            let condvar_c = Arc::clone(&self.condvar);
            let running_c = Arc::clone(&self.running);
            let tasks_c = Arc::clone(&self.tasks);

            self.run_threads.push(thread::spawn(move || {
                // While threads should run
                while *running_c.lock().unwrap() || !tasks_c.lock().unwrap().is_empty() {
                    // Start waiting for signal
                    let (lock, cvar) = &*condvar_c;
                    {
                        let mut started = lock.lock().unwrap();
                        while !*started {
                            started = cvar.wait(started).unwrap();
                        }
                    }

                    // While Tasks left
                    while !tasks_c.lock().unwrap().is_empty() {
                        let mut task: Option<F> = None;

                        // Fetch task.
                        {
                            let mut tasks = tasks_c.lock().unwrap();
                            if !tasks.is_empty() {
                                task = Some(tasks.pop_front().unwrap());
                            }
                        }

                        // Run task.
                        if let Some(current_task) = task {
                            current_task();
                        }
                    }
                    *lock.lock().unwrap() = false;
                }
            }));
        }
    }

    pub fn stop_and_finish(mut self) {
        *self.running.lock().unwrap() = false;

        while !self.run_threads.is_empty() {
            self.run_threads.pop().unwrap().join().unwrap();
        }

        self._notify_one();
    }

    pub fn post(&self, task: F) {
        self.tasks.lock().unwrap().push_back(task);

        self._notify_all();
    }

    pub fn post_timeout(&self, task: F, dur: Duration) {
        thread::sleep(dur);
        self.post(task);
    }

    fn _notify_one(&self) {
        let (lock, cvar) = &*self.condvar;
        *lock.lock().unwrap() = true;
        cvar.notify_one();
    }

    fn _notify_all(&self) {
        let (lock, cvar) = &*self.condvar;
        *lock.lock().unwrap() = true;
        cvar.notify_all();
    }
}
