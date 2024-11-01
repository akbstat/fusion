use std::sync::{mpsc, Arc, Mutex};

use super::{task::ConvertTask, worker::Worker};

pub struct ConvertController {
    status: Option<mpsc::Sender<ConvertTask>>,
    workers: Vec<Worker>,
}

impl ConvertController {
    pub fn new(
        worker_number: usize,
        status: Arc<Mutex<mpsc::Sender<()>>>,
        logger: Arc<Mutex<mpsc::Sender<String>>>,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));
        let mut workers = Vec::with_capacity(worker_number);
        for id in 0..worker_number {
            workers.push(Worker::new(
                id,
                Arc::clone(&logger),
                Arc::clone(&rx),
                Arc::clone(&status),
            ));
        }
        ConvertController {
            status: Some(tx),
            workers,
        }
    }
    pub fn execute(&self, tasks: &[ConvertTask]) {
        let mut tasks = tasks.to_vec();
        tasks.sort_by(|x, y| x.source_size.partial_cmp(&y.source_size).unwrap());
        for task in tasks {
            if let Some(sender) = self.status.as_ref() {
                sender.send(task).ok();
            }
        }
    }
}

impl Drop for ConvertController {
    fn drop(&mut self) {
        drop(self.status.take());
        for worker in &mut self.workers {
            if let Some(handler) = worker.handler() {
                handler.join().ok();
            }
        }
    }
}