use std::{
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
};

use super::worker::PDFCombineWorker;

pub struct PDFCombineController {
    workers: Vec<PDFCombineWorker>,
    sender: Option<mpsc::Sender<(String, PathBuf)>>,
}

impl PDFCombineController {
    pub fn new(
        worker_number: usize,
        status: Arc<Mutex<mpsc::Sender<()>>>,
        logger: Arc<Mutex<mpsc::Sender<String>>>,
        bin: &Path,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));
        let mut workers = Vec::with_capacity(worker_number);
        for id in 0..worker_number {
            let rx = Arc::clone(&rx);
            let status = Arc::clone(&status);
            let logger = Arc::clone(&logger);
            workers.push(PDFCombineWorker::new(id, bin, rx, status, logger));
        }
        PDFCombineController {
            workers,
            sender: Some(tx),
        }
    }

    pub fn combine(&self, configs: &[(String, PathBuf)]) {
        for config in configs {
            if let Some(sender) = self.sender.as_ref() {
                sender.send(config.clone()).ok();
            }
        }
    }
}

impl Drop for PDFCombineController {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            if let Some(worker) = worker.handler() {
                worker.join().unwrap();
            }
        }
    }
}
