use crate::game::world::chunk::Chunk;
use crate::game::world::chunk::ChunkData;
use noise::Perlin;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;

struct ChunkRequest {
    cx: i32,
    cy: i32,
    cz: i32,
}

pub struct ChunkResult {
    cx: i32,
    cy: i32,
    cz: i32,
    pub chunk_data: ChunkData,
}
impl ChunkResult {
    pub fn get_cx(&self) -> i32 {
        self.cx
    }
    pub fn get_cy(&self) -> i32 {
        self.cy
    }
    pub fn get_cz(&self) -> i32 {
        self.cz
    }
}

pub struct ChunkGenerator {
    request_tx: mpsc::Sender<ChunkRequest>,
    result_rx: mpsc::Receiver<ChunkResult>,
    workers: Vec<JoinHandle<()>>,
}

impl ChunkGenerator {
    pub fn new(perlin: Perlin) -> Self {
        let (request_tx, request_rx) = mpsc::channel();
        let (result_tx, result_rx) = mpsc::channel();
        let request_rx = Arc::new(Mutex::new(request_rx));

        let num_workers = num_cpus::get();
        let mut workers = Vec::new();

        for _ in 0..num_workers {
            let rx = request_rx.clone();
            let tx = result_tx.clone();
            let perlin = perlin.clone();

            workers.push(std::thread::spawn(move || {
                worker_loop(rx, tx, perlin);
            }));
        }

        Self {
            request_tx,
            result_rx,
            workers,
        }
    }

    pub fn request(&self, cx: i32, cy: i32, cz: i32) {
        let request = ChunkRequest { cx, cy, cz };
        let _ = self.request_tx.send(request);
    }

    pub fn try_recv(&self) -> Result<ChunkResult, mpsc::TryRecvError> {
        self.result_rx.try_recv()
    }
}

impl Drop for ChunkGenerator {
    fn drop(&mut self) {
        let _ = &mut self.request_tx;
        let workers = std::mem::take(&mut self.workers);
        for worker in workers {
            let _ = worker.join();
        }
    }
}

fn worker_loop(request_rx: Arc<Mutex<mpsc::Receiver<ChunkRequest>>>, result_tx: mpsc::Sender<ChunkResult>, perlin: Perlin) {
    loop {
        let request = {
            let rx = request_rx.lock().unwrap();
            match rx.recv() {
                Ok(req) => req,
                Err(_) => break,
            }
        };
        let chunk = Chunk::generate(request.cx, request.cy, request.cz, &perlin);
        let result = ChunkResult {
            cx: request.cx,
            cy: request.cy,
            cz: request.cz,
            chunk_data: ChunkData::new(chunk),
        };
        let _ = result_tx.send(result);
    }
}
