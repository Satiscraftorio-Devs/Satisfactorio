use std::sync::{mpsc, Arc, Mutex};
use std::thread::{Builder, JoinHandle};

pub struct WorkItem<I, O> {
    pub input: I,
    pub coords: (i32, i32, i32),
    phantom: Option<O>,
}

pub struct WorkResult<O> {
    pub output: O,
    pub coords: (i32, i32, i32),
}

pub trait Parallelizable: Send + 'static {
    type Input: Send + 'static;
    type Output: Send + 'static;
    type Context: Clone + Send + 'static;

    fn process(input: Self::Input, ctx: &Self::Context) -> Self::Output;
}

pub struct WorkerPool<P: Parallelizable> {
    request_tx: mpsc::Sender<WorkItem<P::Input, P::Output>>,
    result_rx: mpsc::Receiver<WorkResult<P::Output>>,
    workers: Vec<JoinHandle<()>>,
    context: P::Context,
}

impl<P: Parallelizable> WorkerPool<P> {
    pub fn new(num_workers: usize, context: P::Context) -> Self {
        let (request_tx, request_rx) = mpsc::channel::<WorkItem<P::Input, P::Output>>();
        let (result_tx, result_rx) = mpsc::channel::<WorkResult<P::Output>>();
        let request_rx = Arc::new(Mutex::new(request_rx));

        let mut workers = Vec::new();

        for worker_id in 0..num_workers {
            let rx = request_rx.clone();
            let tx = result_tx.clone();
            let ctx = context.clone();

            let worker = Builder::new()
                .name(format!("WorkerPool-{}", worker_id))
                .stack_size(8 * 1024 * 1024)
                .spawn(move || loop {
                    let item = {
                        let rx = rx.lock().unwrap();
                        match rx.recv() {
                            Ok(req) => req,
                            Err(_) => break,
                        }
                    };

                    let output = P::process(item.input, &ctx);
                    let result = WorkResult {
                        output,
                        coords: item.coords,
                    };
                    let _ = tx.send(result);
                })
                .expect("Failed to spawn worker thread");

            workers.push(worker);
        }

        Self {
            request_tx,
            result_rx,
            workers,
            context,
        }
    }

    pub fn submit(&self, input: P::Input, coords: (i32, i32, i32)) {
        let item = WorkItem {
            input,
            coords,
            phantom: None,
        };
        let _ = self.request_tx.send(item);
    }

    pub fn try_recv(&self) -> Option<WorkResult<P::Output>> {
        match self.result_rx.try_recv() {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => None,
        }
    }

    pub fn context(&self) -> &P::Context {
        &self.context
    }
}

impl<P: Parallelizable> Drop for WorkerPool<P> {
    fn drop(&mut self) {
        let _ = &mut self.request_tx;

        for worker in std::mem::take(&mut self.workers) {
            let _ = worker.join();
        }
    }
}
